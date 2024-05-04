// use std::process::{Command, Stdio};

use crate::{
    config::{Run, Task},
    console::{Level, Logger},
};

use super::{errors::ExecError, shell, TaskPath};

pub const FALLBACK_EXIT_CODE_FOR_SIGNAL_TERMINATION: i32 = 255;

pub struct ExecOutput {
    pub exit_code: i32,
    pub console_output: Vec<u8>,
}

pub fn handle_execution(
    logger: &Logger,
    task_path: &TaskPath,
    task: &Task,
) -> Result<ExecOutput, ExecError> {
    let shell = shell::resolve_shell(task);

    let expression = match task.run {
        Run::String(ref command) => {
            let shell = shell
                .ok_or_else(|| ExecError::ShellRequired(task_path.clone(), command.clone()))?;

            duct::cmd!(shell, "-c", command)
        }
        Run::Args(ref args) => {
            logger.emit(
                Level::Status,
                format!("running command: {:?}", args.join(" ")),
            );

            duct::cmd(args[0].as_str(), &args[1..])
        }
    };

    let output = expression
        .unchecked()
        .stderr_to_stdout()
        .stdout_capture()
        .run()?;

    Ok(ExecOutput {
        exit_code: output
            .status
            .code()
            .unwrap_or(FALLBACK_EXIT_CODE_FOR_SIGNAL_TERMINATION),
        console_output: output.stdout,
    })
}
