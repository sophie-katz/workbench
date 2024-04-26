use std::iter::Peekable;

use super::{task_args::TaskArgs, workbench_args::ARGS_WITH_VALUES};

#[derive(Debug)]
pub struct SplitArgs {
    pub workbench_args: Vec<String>,
    pub task_args: TaskArgs,
}

pub fn split_args(args: Vec<String>) -> SplitArgs {
    let mut args_iter = args.into_iter().peekable();

    // Get binary name
    let binary_name = args_iter
        .next()
        .expect("args vector must have at least one argument, the binary name");

    // Read workbench arguments
    let workbench_args = collect_workbench_args(binary_name, &mut args_iter);

    // Get task name
    let task_path = read_task_path(&mut args_iter);

    // Get task args
    let task_args = collect_task_args(&mut args_iter);

    SplitArgs {
        workbench_args,
        task_args: TaskArgs {
            task_path,
            task_args,
        },
    }
}

fn collect_workbench_args<T: Iterator<Item = String>>(
    binary_name: String,
    args_iter: &mut Peekable<T>,
) -> Vec<String> {
    let mut wb_args = vec![binary_name];

    loop {
        let arg = args_iter.peek().cloned();

        if let Some(arg) = arg {
            if arg.starts_with("-") {
                wb_args.push(arg.to_string());
                args_iter.next();

                if ARGS_WITH_VALUES.contains(arg.as_str()) {
                    let value = args_iter.next();
                    if let Some(value) = value {
                        wb_args.push(value.to_string());
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    wb_args
}

fn read_task_path<T: Iterator<Item = String>>(args_iter: &mut Peekable<T>) -> Option<String> {
    let task_path = args_iter.next();

    if let Some(ref task_path) = task_path {
        assert!(!task_path.starts_with("-"));
    }

    task_path
}

fn collect_task_args<T: Iterator<Item = String>>(args_iter: &mut Peekable<T>) -> Vec<String> {
    let mut task_args = Vec::new();

    loop {
        let arg = args_iter.next();

        if let Some(arg) = arg {
            task_args.push(arg.to_string());
        } else {
            break;
        }
    }

    task_args
}
