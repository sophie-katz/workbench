use std::{cell::RefCell, rc::Rc};

use crate::{
    config::Config,
    console::{Logger, ProgressContext},
};

use super::TaskPath;

#[derive(Clone)]
pub struct TaskContext {
    pub config: Rc<RefCell<Config>>,
    pub logger: Rc<RefCell<Logger>>,
    pub progress_context: Rc<RefCell<ProgressContext>>,
    pub parents: Vec<TaskPath>,
    pub task_path: TaskPath,
}

impl TaskContext {
    pub fn new(
        config: Rc<RefCell<Config>>,
        logger: Rc<RefCell<Logger>>,
        progress_context: Rc<RefCell<ProgressContext>>,
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

    pub fn next(&self, task_path: TaskPath) -> Self {
        let mut parents = self.parents.clone();
        parents.push(task_path.clone());

        Self {
            config: self.config.clone(),
            logger: self.logger.clone(),
            progress_context: self.progress_context.clone(),
            parents,
            task_path,
        }
    }
}
