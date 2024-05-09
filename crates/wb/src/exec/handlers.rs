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

use std::{
    ffi::OsString,
    time::{Duration, Instant},
};

use shell_quote::QuoteExt;

use crate::{
    config::{Run, Shell, Task},
    error::WorkbenchError,
};

use super::{shell, TaskPath};

pub const FALLBACK_EXIT_CODE_FOR_SIGNAL_TERMINATION: i32 = 255;

#[derive(Debug, PartialEq, Eq)]
pub struct Output {
    pub exit_code: i32,
    pub console_output: Vec<u8>,
    pub duration: Duration,
}

pub fn handle_execution(task_path: &TaskPath, task: &Task) -> Result<Output, WorkbenchError> {
    let expression = match task.run {
        Run::String(ref command) => {
            let shell = shell::resolve(task.shell.as_ref().unwrap_or(&Shell::Bool(true)));

            let shell = shell
                .ok_or_else(|| WorkbenchError::ShellRequired(task_path.clone(), command.clone()))?;

            duct::cmd!(shell, "-c", command)
        }
        Run::Args(ref args) => {
            let shell = shell::resolve(task.shell.as_ref().unwrap_or(&Shell::Bool(false)));

            match shell {
                None => duct::cmd(args[0].as_str(), &args[1..]),
                Some(shell) => {
                    let mut buffer = OsString::new();

                    for arg in args {
                        buffer.push_quoted(shell_quote::Sh, arg.as_str());
                        buffer.push(" ");
                    }

                    duct::cmd!(shell, "-c", buffer)
                }
            }
        }
    };

    let start = Instant::now();

    let output = expression
        .unchecked()
        .stderr_to_stdout()
        .stdout_capture()
        .run()?;

    let duration = start.elapsed();

    Ok(Output {
        exit_code: output
            .status
            .code()
            .unwrap_or(FALLBACK_EXIT_CODE_FOR_SIGNAL_TERMINATION),
        console_output: output.stdout,
        duration,
    })
}

#[cfg(test)]
mod tests {
    use crate::config::Shell;

    use super::*;

    #[test]
    fn test_handle_execution_successful_without_shell_without_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["true".to_string()]),
            shell: None,
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.console_output, vec![]);
    }

    #[test]
    fn test_handle_execution_successful_without_shell_with_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["echo".to_string(), "a".to_string()]),
            shell: None,
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.console_output, vec![b'a', b'\n']);
    }

    #[test]
    fn test_handle_execution_successful_with_shell_without_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["true".to_string()]),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.console_output, vec![]);
    }

    #[test]
    fn test_handle_execution_successful_with_shell_with_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["echo".to_string(), "a".to_string()]),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.console_output, vec![b'a', b'\n']);
    }

    #[test]
    fn test_handle_execution_unsuccessful_without_shell_without_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["false".to_string()]),
            shell: None,
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.console_output, vec![]);
    }

    #[test]
    fn test_handle_execution_unsuccessful_without_shell_with_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec![
                "cat".to_string(),
                "this/file/does/not/exist".to_string(),
            ]),
            shell: None,
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 1);
        assert_eq!(
            output.console_output,
            b"cat: this/file/does/not/exist: No such file or directory\n".to_vec()
        );
    }

    #[test]
    fn test_handle_execution_unsuccessful_with_shell_without_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["false".to_string()]),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.console_output, vec![]);
    }

    #[test]
    fn test_handle_execution_unsuccessful_with_shell_with_output() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec![
                "cat".to_string(),
                "this/file/does/not/exist".to_string(),
            ]),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 1);
        assert_eq!(
            output.console_output,
            b"cat: this/file/does/not/exist: No such file or directory\n".to_vec()
        );
    }

    #[test]
    fn test_handle_execution_stdout_then_stderr() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::String("echo a && echo b 1>&2".to_owned()),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.console_output, b"a\nb\n".to_vec());
    }

    #[test]
    fn test_handle_execution_stderr_then_stdout() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::String("echo a 1>&2 && echo b".to_owned()),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.console_output, b"a\nb\n".to_vec());
    }

    // #[test]
    // fn test_handle_execution_args_with_spaces_without_shell() {
    //     let task_path = TaskPath::parse("a").unwrap();

    //     let task = Task {
    //         run: Run::Args(vec![
    //             "cat".to_owned(),
    //             "this path does not exist".to_owned(),
    //         ]),
    //         shell: None,
    //         dependencies: None,
    //         inputs: None,
    //         outputs: None,
    //         description: None,
    //         examples: None,
    //         usage: None,
    //     };

    //     let output = handle_execution(&task_path, &task).unwrap();

    //     assert_eq!(output.exit_code, 1);
    //     assert_eq!(
    //         output.console_output,
    //         b"cat: this path does not exist: No such file or directory\n".to_vec()
    //     );
    // }

    // #[test]
    // fn test_handle_execution_args_with_spaces_with_shell() {
    //     let task_path = TaskPath::parse("a").unwrap();

    //     let task = Task {
    //         run: Run::Args(vec![
    //             "cat".to_owned(),
    //             "this path does not exist".to_owned(),
    //         ]),
    //         shell: Some(Shell::Bool(true)),
    //         dependencies: None,
    //         inputs: None,
    //         outputs: None,
    //         description: None,
    //         examples: None,
    //         usage: None,
    //     };

    //     let output = handle_execution(&task_path, &task).unwrap();

    //     assert_eq!(output.exit_code, 1);
    //     assert_eq!(
    //         output.console_output,
    //         b"cat: this path does not exist: No such file or directory\n".to_vec()
    //     );
    // }

    #[test]
    fn test_handle_execution_string_with_shell() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::String("set".to_owned()),
            shell: Some(Shell::Bool(true)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        assert_eq!(handle_execution(&task_path, &task).unwrap().exit_code, 0);
    }

    #[test]
    fn test_handle_execution_vec_without_shell() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec!["echo".to_owned(), "$SHELL".to_owned()]),
            shell: None,
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        assert_eq!(
            handle_execution(&task_path, &task).unwrap().console_output,
            b"$SHELL\n".to_vec()
        );
    }

    #[test]
    fn test_handle_execution_vec_with_shell() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::Args(vec![":".to_owned()]),
            shell: Some(Shell::String("/bin/sh".to_owned())),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        let output = handle_execution(&task_path, &task).unwrap();

        assert_eq!(output.exit_code, 0);
        assert!(output.console_output.is_empty());
    }

    #[test]
    fn test_handle_execution_string_defaults_to_shell() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::String("set".to_owned()),
            shell: None,
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        assert_eq!(handle_execution(&task_path, &task).unwrap().exit_code, 0);
    }

    #[test]
    fn test_handle_execution_string_with_shell_false() {
        let task_path = TaskPath::parse("a").unwrap();

        let task = Task {
            run: Run::String("set".to_owned()),
            shell: Some(Shell::Bool(false)),
            dependencies: None,
            inputs: None,
            outputs: None,
            description: None,
            examples: None,
            usage: None,
        };

        assert!(handle_execution(&task_path, &task).is_err());
    }
}
