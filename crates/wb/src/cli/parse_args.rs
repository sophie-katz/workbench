// Copyright 2024 Sophie Katz
//
// This file is part of Workbench.
//
// Workbench is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Workbench is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Workbench. If not,
// see <https://www.gnu.org/licenses/>.

use std::env;

use clap::Parser;

use super::{split_args, TaskArgs, WorkbenchArgs};

pub fn parse_args() -> (WorkbenchArgs, TaskArgs) {
    parse_args_from_vec(env::args().collect())
}

fn parse_args_from_vec(vec: Vec<String>) -> (WorkbenchArgs, TaskArgs) {
    let (workbench_args, task_args) = split_args::split_args(vec);

    (WorkbenchArgs::parse_from(workbench_args), task_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_parse_args_from_vec_empty() {
        parse_args_from_vec(vec![]);
    }

    #[test]
    fn test_parse_args_from_vec_wb() {
        assert_eq!(
            parse_args_from_vec(vec!["wb".to_owned()]),
            (
                WorkbenchArgs {
                    jobs: 0,
                    verbose: false,
                    config: None,
                    disable_progress: false,
                    disable_color: false,
                    disable_unicode: false,
                },
                TaskArgs {
                    target_task_path: None,
                    task_args: Vec::new()
                }
            )
        );
    }

    #[test]
    fn test_parse_args_from_vec_wb_j_1() {
        assert_eq!(
            parse_args_from_vec(vec!["wb".to_owned(), "-j".to_owned(), "1".to_owned()]),
            (
                WorkbenchArgs {
                    jobs: 1,
                    verbose: false,
                    config: None,
                    disable_progress: false,
                    disable_color: false,
                    disable_unicode: false,
                },
                TaskArgs {
                    target_task_path: None,
                    task_args: Vec::new()
                }
            )
        );
    }

    #[test]
    fn test_parse_args_from_vec_wb_a() {
        assert_eq!(
            parse_args_from_vec(vec!["wb".to_owned(), "a".to_owned()]),
            (
                WorkbenchArgs {
                    jobs: 0,
                    verbose: false,
                    config: None,
                    disable_progress: false,
                    disable_color: false,
                    disable_unicode: false,
                },
                TaskArgs {
                    target_task_path: Some("a".to_owned()),
                    task_args: Vec::new()
                }
            )
        );
    }

    #[test]
    fn test_parse_args_from_vec_wb_jobs_1_a() {
        assert_eq!(
            parse_args_from_vec(vec![
                "wb".to_owned(),
                "--jobs".to_owned(),
                "1".to_owned(),
                "a".to_owned()
            ]),
            (
                WorkbenchArgs {
                    jobs: 1,
                    verbose: false,
                    config: None,
                    disable_progress: false,
                    disable_color: false,
                    disable_unicode: false,
                },
                TaskArgs {
                    target_task_path: Some("a".to_owned()),
                    task_args: Vec::new()
                }
            )
        );
    }

    #[test]
    fn test_parse_args_from_vec_wb_jobs_1_a_b_c() {
        assert_eq!(
            parse_args_from_vec(vec![
                "wb".to_owned(),
                "--jobs".to_owned(),
                "1".to_owned(),
                "a".to_owned(),
                "b".to_owned(),
                "c".to_owned()
            ]),
            (
                WorkbenchArgs {
                    jobs: 1,
                    verbose: false,
                    config: None,
                    disable_progress: false,
                    disable_color: false,
                    disable_unicode: false,
                },
                TaskArgs {
                    target_task_path: Some("a".to_owned()),
                    task_args: vec!["b".to_owned(), "c".to_owned()]
                }
            )
        );
    }
}
