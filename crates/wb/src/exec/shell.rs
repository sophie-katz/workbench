use crate::config::{Shell, Task};

const DEFAULT_SHELL: &str = "sh";

pub fn resolve_shell(task: &Task) -> Option<String> {
    match task.shell {
        None => None,
        Some(Shell::Bool(true)) => Some(DEFAULT_SHELL.to_owned()),
        Some(Shell::Bool(false)) => None,
        Some(Shell::String(ref shell)) => Some(shell.clone()),
    }
}
