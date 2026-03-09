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
    /// Sync local repository with a git bundle file
    #[command(
        long_about = "Update local branches to match the bundle. Removes branches not in bundle, adds new branches, and updates existing ones."
    )]
    Sync(Sync),

    /// Push local repository to a remote server
    #[command(
        long_about = "Create a git bundle from all local branches, transfer it to the remote server via SCP, and run sync on the remote."
    )]
    Push(Push),

    /// Pull repository from a remote server
    #[command(
        long_about = "Create a git bundle on the remote server, transfer it locally via SCP, and sync local branches."
    )]
    Pull(Pull),
}

/// A CLI tool for syncing git repositories between local and remote servers using git bundles.
/// Useful for air-gapped environments or restricted networks without direct git remote access.
#[derive(Parser)]
#[command(version, verbatim_doc_comment)]
struct Cli {
    /// Generate shell completion script
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
