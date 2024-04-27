use std::rc::Rc;

use crate::{
    config::Config,
    console::{Logger, ProgressContext},
};

use super::TaskPath;

#[derive(Clone)]
pub struct TaskContext<'config, 'logger> {
    pub config: &'config Config,
    pub logger: &'logger Logger,
    pub progress_context: Rc<ProgressContext>,
    pub parents: Vec<TaskPath>,
    pub task_path: TaskPath,
}

impl<'config, 'logger> TaskContext<'config, 'logger> {
    pub fn new(
        config: &'config Config,
        logger: &'logger Logger,
        progress_context: Rc<ProgressContext>,
        task_path: TaskPath,
    ) -> Self {
        Self {
            config,
            logger,
            progress_context,
            parents: Vec::new(),
            task_path,
        }
    }

    pub fn next(&mut self, task_path: TaskPath) -> Self {
        let mut parents = self.parents.clone();
        parents.push(task_path.clone());

        Self {
            config: self.config,
            logger: self.logger,
            progress_context: self.progress_context.clone(),
            parents,
            task_path,
        }
    }
}
