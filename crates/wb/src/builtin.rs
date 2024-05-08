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

mod task_ls;

use colored::Colorize;

use crate::{
    config::Config,
    console::{Level, Log},
    exec::{get_task_at_path, resolve_paths, TaskPath},
};

pub fn try_exec_builtin_task(config: &Config, target_task_path: &TaskPath) -> Option<bool> {
    if !target_task_path.built_in {
        return None;
    }

    assert!(
        target_task_path.namespace.is_none(),
        "namespace is not supported for built-in tasks"
    );

    Some(
        match (
            target_task_path.name.as_str(),
            target_task_path.property.as_ref(),
        ) {
            ("ls", None) => task_ls::exec(config),
            _ => false,
        },
    )
}

pub fn try_exec_builtin_property(
    logger: &impl Log,
    config: &Config,
    target_task_path: &TaskPath,
) -> Option<bool> {
    let task = get_task_at_path(config, target_task_path)?;

    let property = target_task_path.property.as_ref()?.as_str();

    match property {
        "help" => {
            let target_task_path_without_property = TaskPath {
                namespace: target_task_path.namespace.clone(),
                built_in: target_task_path.built_in,
                name: target_task_path.name.clone(),
                property: None,
            };

            print!(
                "{} {} ",
                "usage:".bold().dimmed().white(),
                "wb".green().bold()
            );

            if let Some(usage) = &task.usage {
                println!("{} {}", target_task_path_without_property, usage);
            } else {
                println!(
                    "{} {}",
                    target_task_path_without_property,
                    "...".dimmed().white()
                );
            }

            if let Some(description) = &task.description {
                println!();
                println!("{}", description);
            }

            if let Some(examples) = &task.examples {
                println!();
                println!("{}", "examples:".bold().dimmed().white());

                let mut first = true;

                for example in examples {
                    if first {
                        first = false;
                    } else {
                        println!();
                    }

                    if let Some(description) = &example.description {
                        println!("  {} {}", "#".magenta(), description.magenta());
                    }

                    println!(
                        "  {} {} {} {}",
                        "$".dimmed().white(),
                        "wb".green().bold(),
                        target_task_path_without_property,
                        example.run
                    );
                }
            }

            Some(true)
        }
        "description" => {
            if let Some(description) = &task.description {
                println!("{}", description);
                Some(true)
            } else {
                logger.log_message(
                    Level::Error,
                    format!("task does not have a description set"),
                );
                Some(false)
            }
        }
        "resolved-inputs" => {
            if let Some(inputs) = &task.inputs {
                let paths = match resolve_paths(inputs) {
                    Ok(paths) => paths,
                    Err(err) => {
                        logger.log_message(
                            Level::Error,
                            format!("error while resolving task inputs: {err}"),
                        );
                        return Some(false);
                    }
                };

                for path in paths {
                    println!("{}", path.display());
                }
            } else {
                logger.log_message(Level::Warning, "task does not have any inputs defined");
            }

            Some(true)
        }
        "resolved-outputs" => {
            if let Some(outputs) = &task.outputs {
                let paths = match resolve_paths(outputs) {
                    Ok(paths) => paths,
                    Err(err) => {
                        logger.log_message(
                            Level::Error,
                            format!("error while resolving task outputs: {err}"),
                        );
                        return Some(false);
                    }
                };

                for path in paths {
                    println!("{}", path.display());
                }
            } else {
                logger.log_message(Level::Warning, "task does not have any outputs defined");
            }

            Some(true)
        }
        _ => None,
    }
}
