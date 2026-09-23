use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rusty",
    version,
    about = "Project development tooling for the Topcoat Rust template"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    #[command(
        name = "migration",
        visible_alias = "make:migration",
        about = "Create a SQLx-compatible migration"
    )]
    Migration(MigrationArgs),

    #[command(name = "doctor", about = "Inspect the Rusty CLI project environment")]
    Doctor,
}

#[derive(Debug, Args)]
pub(crate) struct MigrationArgs {
    /// Migration description, e.g. CreateContacts or create_contacts.
    pub(crate) name: String,

    /// Create one irreversible SQL migration instead of an up/down pair.
    #[arg(long)]
    pub(crate) simple: bool,
}
