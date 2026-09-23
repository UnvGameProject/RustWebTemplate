mod doctor;
mod migration;

use std::error::Error;

use crate::cli::Command;

pub(crate) type CommandResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) fn run(command: Command) -> CommandResult {
    match command {
        Command::Migration(args) => migration::run(args),
        Command::Doctor => doctor::run(),
    }
}
