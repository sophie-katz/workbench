mod cli;
mod config;
mod exec;

use exec::TaskPath;
use std::env;

fn main() {
    let (workbench_args, task_args) = cli::parse_args();

    println!("workbench args: {:?}", workbench_args);
    println!("task args: {:?}", task_args);

    let config_path = config::resolve_path(&env::current_dir().unwrap(), None).unwrap();
    let config = config::load(&config_path).unwrap();

    println!("config: {:?}", config);

    exec::exec(
        &config,
        TaskPath {
            namespace: None,
            name: "b".to_owned(),
            property: None,
        },
    )
    .unwrap();
}
