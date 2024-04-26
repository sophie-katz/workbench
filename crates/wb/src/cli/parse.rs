use std::env;

use clap::Parser;

use super::{split_args, TaskArgs, WorkbenchArgs};

pub fn parse_args() -> (WorkbenchArgs, TaskArgs) {
    let split_args = split_args::split_args(env::args().collect());

    let workbench_args = WorkbenchArgs::parse_from(split_args.workbench_args);

    (workbench_args, split_args.task_args)
}
