use std::io::{self, Error as IoError};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use pull::{Pull, PullError};
use push::{Push, PushError};
use sync::{Sync, SyncError};
use thiserror::Error;

mod process;
mod pull;
mod push;
mod sync;

#[derive(Error, Debug)]
enum MainError {
    #[error("IoError: {0}")]
    Io(#[from] IoError),
    #[error("SyncError: {0}")]
    Sync(#[from] SyncError),
    #[error("PushError: {0}")]
    Push(#[from] PushError),
    #[error("PullError: {0}")]
    Pull(#[from] PullError),
}

#[derive(Subcommand)]
enum Commands {
    /// Sync code with git bundle file
    Sync(Sync),
    /// Push code to remote server with git bundle file
    Push(Push),
    /// Pull code from remote server with git bundle file
    Pull(Pull),
}

#[derive(Parser)]
struct Cli {
    /// Generate shell completion
    #[arg(long, value_enum)]
    generate: Option<Shell>,
    #[command(subcommand)]
    commands: Option<Commands>,
}

fn main() -> Result<(), MainError> {
    let cli = Cli::parse();

    if let Some(generator) = cli.generate {
        let cmd = &mut Cli::command();

        generate(
            generator,
            cmd,
            cmd.get_name().to_string(),
            &mut io::stdout(),
        );
    } else {
        match cli.commands {
            Some(Commands::Sync(mut sync)) => sync.run()?,
            Some(Commands::Push(push)) => push.run()?,
            Some(Commands::Pull(pull)) => pull.run()?,
            _ => Cli::command().print_help()?,
        }
    }

    Ok(())
}
