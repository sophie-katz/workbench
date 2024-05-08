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

use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    config::{Config, Task},
    error::WorkbenchError,
};

lazy_static! {
    static ref TASK_PATH_REGEX: Regex =
        Regex::new(r"^(?<namespace>[\pL-]*:)?(?<name>[\pL-]+)(?<property>\.[\pL-]+)?$").unwrap();
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TaskPath {
    pub namespace: Option<String>,
    pub built_in: bool,
    pub name: String,
    pub property: Option<String>,
}

impl TaskPath {
    pub fn parse(text: &str) -> Result<Self, WorkbenchError> {
        if let Some(captures) = TASK_PATH_REGEX.captures(text) {
            let namespace = captures
                .name("namespace")
                .map(|m| m.as_str())
                .filter(|m| !m.is_empty())
                .map(ToOwned::to_owned);

            let built_in = match namespace {
                Some(ref namespace) => namespace == ":",
                None => false,
            };

            let namespace = namespace
                .map(|namespace| {
                    if namespace.ends_with(':') {
                        namespace[..namespace.len() - 1].to_owned()
                    } else {
                        namespace
                    }
                })
                .filter(|namespace| !namespace.is_empty());

            let name = captures
                .name("name")
                .map(|m| m.as_str())
                .filter(|m| !m.is_empty())
                .expect("name is required")
                .to_owned();

            let property = captures
                .name("property")
                .map(|m| m.as_str())
                .filter(|m| !m.is_empty())
                .map(|m| {
                    if let Some(end) = m.strip_prefix('.') {
                        end.to_owned()
                    } else {
                        m.to_owned()
                    }
                });

            Ok(Self {
                namespace,
                built_in,
                name,
                property,
            })
        } else {
            Err(WorkbenchError::InvalidTaskPath(text.to_owned()))
        }
    }
}

impl Display for TaskPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref namespace) = self.namespace {
            write!(f, "{namespace}:")?;
        } else if self.built_in {
            write!(f, ":")?;
        }

        write!(f, "{}", self.name)?;

        if let Some(ref property) = self.property {
            write!(f, ".{property}")?;
        }

        Ok(())
    }
}

pub fn get_task_at_path<'config>(
    config: &'config Config,
    task_path: &TaskPath,
) -> Option<&'config Task> {
    if let Some(namespace) = &task_path.namespace {
        if let Some(ref config_namespaces) = config.namespaces {
            config_namespaces
                .get(namespace.as_str())?
                .tasks
                .get(&task_path.name)
        } else {
            None
        }
    } else if let Some(ref config_tasks) = config.tasks {
        config_tasks.get(&task_path.name)
    } else {
        None
    }
}

pub fn count_dependencies_of_path(
    config: &Config,
    task_path: &TaskPath,
) -> Result<Option<u64>, WorkbenchError> {
    Ok(match get_task_at_path(config, task_path) {
        Some(task) => {
            let mut count: u64 = 0;

            if let Some(ref dependencies) = task.dependencies {
                count += dependencies.len() as u64;

                for dependency in dependencies {
                    match count_dependencies_of_path(
                        config,
                        &TaskPath::parse(dependency.as_str())?,
                    )? {
                        Some(dependencies_count) => {
                            count += dependencies_count;
                        }
                        None => return Ok(None),
                    }
                }
            }

            Some(count)
        }
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_path_parse_empty() {
        assert!(TaskPath::parse("").is_err());
    }

    #[test]
    fn test_task_path_parse_name_only() {
        assert_eq!(
            TaskPath::parse("a").unwrap(),
            TaskPath {
                namespace: None,
                built_in: false,
                name: "a".to_owned(),
                property: None,
            }
        );
    }

    #[test]
    fn test_task_path_parse_built_in() {
        assert_eq!(
            TaskPath::parse(":a").unwrap(),
            TaskPath {
                namespace: None,
                built_in: true,
                name: "a".to_owned(),
                property: None,
            }
        );
    }

    #[test]
    fn test_task_path_parse_namespace() {
        assert_eq!(
            TaskPath::parse("a:b").unwrap(),
            TaskPath {
                namespace: Some("a".to_owned()),
                built_in: false,
                name: "b".to_owned(),
                property: None,
            }
        );
    }

    #[test]
    fn test_task_path_parse_property() {
        assert_eq!(
            TaskPath::parse("a.b").unwrap(),
            TaskPath {
                namespace: None,
                built_in: false,
                name: "a".to_owned(),
                property: Some("b".to_owned()),
            }
        );
    }

    #[test]
    fn test_task_path_parse_illegal() {
        assert!(TaskPath::parse("a.b`").is_err());
    }
}
