use std::{cell::RefCell, rc::Rc};

use crate::{config::Config, exec::TaskPath};

pub struct ProgressContext {}

impl ProgressContext {
    pub fn new(config: Rc<RefCell<Config>>, task_path: TaskPath) -> Self {
        Self {}
    }
}
