use std::path::Path;

use crate::{commands::CommandResult, project::Project};

pub(crate) fn run() -> CommandResult {
    let project = Project::discover()?;
    let migrations = project.migrations_dir();

    println!("Rusty CLI");
    println!();
    println!("project root : {}", project.root().display());
    println!("migrations   : {}", migrations.display());
    println!(
        "migrator key : {}",
        status(Path::new("/run/secrets/postgres_migrator_password"))
    );

    Ok(())
}

fn status(path: &Path) -> &'static str {
    if path.is_file() {
        "available"
    } else {
        "not mounted"
    }
}
