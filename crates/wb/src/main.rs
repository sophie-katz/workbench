mod cli;
mod config;
mod console;
mod exec;

use exec::TaskPath;
use std::{cell::RefCell, env, rc::Rc};

use console::{Level, Logger};

fn main() {
    let (workbench_args, task_args) = cli::parse_args();

    let logger = Rc::new(RefCell::new(Logger {
        min_level: match workbench_args.verbose {
            true => console::Level::Status,
            false => console::Level::Warning,
        },
    }));

    logger.borrow().emit(
        Level::Status,
        format!("workbench arguments: {:?}", workbench_args),
    );

    logger
        .borrow()
        .emit(Level::Status, format!("task arguments: {:?}", task_args));

    let config_path = config::resolve_path(&env::current_dir().unwrap(), None).unwrap();
    let config = Rc::new(RefCell::new(config::load(&config_path).unwrap()));

    logger
        .borrow()
        .emit(Level::Status, format!("loaded configuration: {:?}", config));

    let task_path = TaskPath::parse(task_args.task_path.unwrap().as_str()).unwrap();

    match exec::exec(config, logger.clone(), task_path) {
        Ok(_) => logger
            .borrow()
            .emit(Level::Status, "tasks completed successfully"),
        Err(e) => logger.borrow().emit(Level::Error, e.to_string()),
    }
}
