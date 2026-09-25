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

    #[command(
        name = "make:test",
        visible_alias = "test",
        about = "Scaffold a module-local Rust test file"
    )]
    MakeTest(TestArgs),

    #[command(
        name = "make:command",
        about = "Scaffold and register a new Rusty CLI command"
    )]
    MakeCommand(CommandArgs),

    #[command(name = "doctor", about = "Inspect the Rusty CLI project environment")]
    Doctor,

    #[command(
        name = "audit",
        about = "Audit application source for security policy violations"
    )]
    Audit,
    // RUSTY_COMMAND_VARIANTS
}

#[derive(Debug, Args)]
pub(crate) struct MigrationArgs {
    /// Migration description, e.g. CreateContacts or create_contacts.
    pub(crate) name: String,

    /// Create one irreversible SQL migration instead of an up/down pair.
    #[arg(long)]
    pub(crate) simple: bool,
}

#[derive(Debug, Args)]
pub(crate) struct TestArgs {
    /// Module path relative to src/, e.g. db/models/contact or web/pages/contacts.
    pub(crate) module: String,

    /// Test file name, e.g. persistence or ContactPersistence.
    ///
    /// Omit this when using --view.
    #[arg(required_unless_present = "view")]
    pub(crate) name: Option<String>,

    /// Create tests.rs for a file-backed view module instead of a directory module.
    ///
    /// Example: rusty make:test --view web/pages/contacts
    #[arg(long, conflicts_with = "name")]
    pub(crate) view: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CommandArgs {
    /// Command name, e.g. model:sync, docs:manifest, or HelloWorld.
    pub(crate) name: String,

    /// Show what would be generated without writing files.
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[cfg(test)]
mod tests;
