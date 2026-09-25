mod doctor;
mod make_command;
mod migration;
mod test;

mod audit;

// RUSTY_COMMAND_MODULES

use std::error::Error;

use crate::cli::Command;

pub(crate) type CommandResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) fn run(command: Command) -> CommandResult {
    match command {
        Command::Migration(args) => migration::run(args),
        Command::MakeTest(args) => test::run(args),
        Command::MakeCommand(args) => make_command::run(args),
        Command::Doctor => doctor::run(),
        Command::Audit => audit::run(),
        // RUSTY_COMMAND_DISPATCH
    }
}
