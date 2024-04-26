use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;

use crate::config::{Config, Task};

lazy_static! {
    static ref TASK_PATH_REGEX: Regex =
        Regex::new(r"^(?<namespace>\pL*:)?(?<name>\pL+)(?<property>\.\pL+)?$").unwrap();
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TaskPath {
    pub namespace: Option<String>,
    pub name: String,
    pub property: Option<String>,
}

impl TaskPath {
    pub fn parse(text: &str) -> Option<TaskPath> {
        if let Some(captures) = TASK_PATH_REGEX.captures(text) {
            let namespace = captures
                .name("namespace")
                .map(|m| m.as_str())
                .filter(|m| !m.is_empty())
                .map(|m| m.to_owned());

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
                .map(|m| m.to_owned());

            Some(TaskPath {
                namespace,
                name,
                property,
            })
        } else {
            None
        }
    }
}

impl Display for TaskPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref namespace) = self.namespace {
            write!(f, "{}:", namespace)?;
        }

        write!(f, "{}", self.name)?;

        if let Some(ref property) = self.property {
            write!(f, ".{}", property)?;
        }

        Ok(())
    }
}

pub fn get_task_at_path<'config>(
    config: &'config Config,
    task_path: &TaskPath,
) -> Option<&'config Task> {
    if let Some(path_namespace) = &task_path.namespace {
        if let Some(ref config_namespaces) = config.namespaces {
            config_namespaces
                .get(path_namespace.as_str())?
                .tasks
                .get(&task_path.name)
        } else {
            None
        }
    } else {
        if let Some(ref config_tasks) = config.tasks {
            config_tasks.get(&task_path.name)
        } else {
            None
        }
    }
}
