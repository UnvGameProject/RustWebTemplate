mod cli;
mod commands;
mod project;

use clap::Parser;

use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = commands::run(cli.command) {
        eprintln!("rusty: {error}");
        std::process::exit(1);
    }
}
