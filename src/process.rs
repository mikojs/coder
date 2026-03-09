use std::{env, io::Error as IoError, process::Command, string::FromUtf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("IoError: {0}")]
    Io(#[from] IoError),
    #[error("FromUtf8Error: {0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("Couldn't find the command: {0}")]
    CommandNotFound(String),
    #[error("Run command fails")]
    RunCommandFails,
}

fn command_exist(command: &str) -> bool {
    Command::new(command).output().is_ok()
}

pub fn exec(command: &str, args: Vec<&str>) -> Result<(), ProcessError> {
    if !command_exist(command) {
        return Err(ProcessError::CommandNotFound(command.to_string()));
    }

    if env::var("CODER_DEBUG").is_ok() {
        if !Command::new(command).args(args).status()?.success() {
            return Err(ProcessError::RunCommandFails);
        }
    } else {
        Command::new(command).args(args).output()?;
    }

    Ok(())
}

pub fn exec_result(command: &str, args: Vec<&str>) -> Result<String, ProcessError> {
    if !command_exist(command) {
        return Err(ProcessError::CommandNotFound(command.to_string()));
    }

    Ok(String::from_utf8(
        Command::new(command).args(args).output()?.stdout,
    )?)
}
