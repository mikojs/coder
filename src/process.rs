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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_result_with_args() {
        let result = exec_result("echo", vec!["hello", "world"]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");
    }

    #[test]
    fn test_exec_success() {
        let result = exec("echo", vec!["test"]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_exec_command_not_found() {
        let result = exec("nonexistent_command_12345", vec![]);

        assert!(result.is_err());

        if let Err(ProcessError::CommandNotFound(cmd)) = result {
            assert_eq!(cmd, "nonexistent_command_12345");
        }
    }

    #[test]
    fn test_exec_result_command_not_found() {
        let result = exec_result("nonexistent_command_12345", vec![]);

        assert!(result.is_err());

        if let Err(ProcessError::CommandNotFound(cmd)) = result {
            assert_eq!(cmd, "nonexistent_command_12345");
        }
    }
}
