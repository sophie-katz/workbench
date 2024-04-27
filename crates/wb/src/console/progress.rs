use crate::{config::Config, exec::TaskPath};

pub struct ProgressContext {}

impl ProgressContext {
    pub fn new(config: &Config, task_path: TaskPath) -> Self {
        Self {}
    }
}
