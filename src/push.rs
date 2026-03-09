use std::{fs::remove_file, io::Error as IoError, path::PathBuf};

use clap::Args;
use thiserror::Error;
use url::Url;

use crate::process::{exec, exec_result, ProcessError};

#[derive(Error, Debug)]
pub enum PushError {
    #[error("IoError: {0}")]
    Io(#[from] IoError),
    #[error("ProcessError: {0}")]
    Process(#[from] ProcessError),
}

#[derive(Args)]
pub struct Push {
    /// The ssh URL of the remote server (ex: ssh://user@host)
    ssh_url: Url,
    /// The target repository directory in the remote server
    directory: PathBuf,
}

impl Push {
    pub fn run(&self) -> Result<(), PushError> {
        let ssh_url = self.ssh_url.as_str().replace("ssh://", "");

        println!("Creating bundle file");
        exec("git", vec!["bundle", "create", "temp.bundle", "--all"])?;

        let git_root_path = exec_result(
            "ssh",
            vec![
                &ssh_url,
                &format!(
                    "cd {} && git rev-parse --show-toplevel",
                    self.directory.display()
                ),
            ],
        )?;
        let bundle_dir = PathBuf::from(git_root_path.trim()).join("..");
        let file_path = bundle_dir.join("temp.bundle");

        println!("Pushing bundle file");
        exec(
            "scp",
            vec![
                "temp.bundle",
                &format!("{}:{}", ssh_url, bundle_dir.display()),
            ],
        )?;

        println!("Syncing the repository");
        exec(
            "ssh",
            vec![
                &ssh_url,
                &format!(
                    "cd {} && coder sync {} && rm {}",
                    self.directory.display(),
                    file_path.display(),
                    file_path.display(),
                ),
            ],
        )?;
        remove_file("temp.bundle")?;

        Ok(())
    }
}
