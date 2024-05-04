mod cli;
mod config;
mod console;
mod exec;

use console::{Level, Logger};
use exec::TaskPath;
use std::{env, process::exit};

fn main() {
    let (workbench_args, task_args) = cli::parse_args();

    let logger = Logger {
        min_level: match workbench_args.verbose {
            true => console::Level::Status,
            false => console::Level::Warning,
        },
    };

    logger.emit(
        Level::Status,
        format!("workbench arguments: {:?}", workbench_args),
    );

    logger.emit(Level::Status, format!("task arguments: {:?}", task_args));

    let config_path = config::resolve_path(&env::current_dir().unwrap(), None).unwrap();
    let config = config::load(&config_path).unwrap();

    logger.emit(Level::Status, format!("loaded configuration: {:?}", config));

    let target_task_path = TaskPath::parse(task_args.target_task_path.unwrap().as_str()).unwrap();

    match exec::exec(config, logger.clone(), target_task_path) {
        Ok(succeeded) => {
            if !succeeded {
                exit(1);
            }
        }
        Err(e) => {
            logger.emit(Level::Error, e.to_string());
            exit(1);
        }
    }
}
