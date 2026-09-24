use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::Path,
};

use convert_case::{Case, Casing};

use crate::{cli::CommandArgs, commands::CommandResult, project::Project};

const VARIANT_MARKER: &str = "    // RUSTY_COMMAND_VARIANTS";

const MODULE_MARKER: &str = "// RUSTY_COMMAND_MODULES";

const DISPATCH_MARKER: &str = "        // RUSTY_COMMAND_DISPATCH";

pub(crate) fn run(args: CommandArgs) -> CommandResult {
    let project = Project::discover()?;

    let names = CommandNames::parse(&args.name)?;

    let rusty_src = project.root().join("tools/rusty-cli/src");

    let cli_path = rusty_src.join("cli.rs");

    let commands_dir = rusty_src.join("commands");
    let commands_mod_path = commands_dir.join("mod.rs");

    let command_path = commands_dir.join(format!("{}.rs", names.module));

    let cli_contents = fs::read_to_string(&cli_path)?;
    let commands_mod_contents = fs::read_to_string(&commands_mod_path)?;

    validate_markers(&cli_contents, &commands_mod_contents)?;

    ensure_not_registered(&names, &cli_contents, &commands_mod_contents, &command_path)?;

    let command_source = generate_command(&names);

    let updated_cli = insert_before(&cli_contents, VARIANT_MARKER, &generate_variant(&names))?;

    let updated_commands_mod = insert_before(
        &insert_before(
            &commands_mod_contents,
            MODULE_MARKER,
            &format!("mod {};", names.module),
        )?,
        DISPATCH_MARKER,
        &format!(
            "        Command::{} => {}::run(),",
            names.variant, names.module,
        ),
    )?;

    if args.dry_run {
        println!("Rusty command scaffold preview");
        println!();
        println!("command : {}", names.cli);
        println!("variant : {}", names.variant);
        println!("module  : {}", names.module);
        println!("file    : {}", command_path.display());
        println!();
        println!("No files changed.");

        return Ok(());
    }

    write_new(&command_path, &command_source)?;

    if let Err(error) = fs::write(&cli_path, updated_cli) {
        let _ = fs::remove_file(&command_path);
        return Err(error.into());
    }

    if let Err(error) = fs::write(&commands_mod_path, updated_commands_mod) {
        let _ = fs::remove_file(&command_path);

        return Err(io::Error::other(format!(
            "command module was removed after registry update failed; \
             inspect {} before retrying: {error}",
            cli_path.display(),
        ))
        .into());
    }

    println!("Created Rusty command:");
    println!("  {}", command_path.display());
    println!();
    println!("Registered:");
    println!("  command : {}", names.cli);
    println!("  variant : {}", names.variant);
    println!("  module  : {}", names.module);
    println!();
    println!("Updated:");
    println!("  {}", cli_path.display());
    println!("  {}", commands_mod_path.display());

    Ok(())
}

struct CommandNames {
    cli: String,
    module: String,
    variant: String,
}

impl CommandNames {
    fn parse(raw: &str) -> io::Result<Self> {
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "command name cannot be empty",
            ));
        }

        let segments: Vec<String> = raw
            .split(':')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.to_case(Case::Kebab))
            .collect();

        if segments.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "command name is invalid",
            ));
        }

        for segment in &segments {
            validate_cli_segment(segment)?;
        }

        let cli = segments.join(":");

        let identifier_source = segments.join("_");

        let module = identifier_source.to_case(Case::Snake);

        let variant = identifier_source.to_case(Case::Pascal);

        validate_rust_identifier(&module, "generated module name")?;

        validate_rust_identifier(&variant, "generated enum variant")?;

        Ok(Self {
            cli,
            module,
            variant,
        })
    }
}

fn generate_variant(names: &CommandNames) -> String {
    format!(
        "    #[command(\n\
         \x20       name = \"{}\",\n\
         \x20       about = \"TODO: describe the {} command\"\n\
         \x20   )]\n\
         \x20   {},",
        names.cli, names.cli, names.variant,
    )
}

fn generate_command(names: &CommandNames) -> String {
    format!(
        "\
use crate::commands::CommandResult;

pub(crate) fn run() -> CommandResult {{
    println!(\"{} executed\");

    Ok(())
}}
",
        names.cli,
    )
}

fn ensure_not_registered(
    names: &CommandNames,
    cli: &str,
    commands_mod: &str,
    command_path: &Path,
) -> io::Result<()> {
    if command_path.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("command module already exists: {}", command_path.display()),
        ));
    }

    let command_needle = format!("name = \"{}\"", names.cli);

    if cli.contains(&command_needle) {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("CLI command is already registered: {}", names.cli),
        ));
    }

    let variant_needle = format!("Command::{}", names.variant);

    if commands_mod.contains(&variant_needle) {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("command variant is already dispatched: {}", names.variant),
        ));
    }

    Ok(())
}

fn validate_markers(cli: &str, commands_mod: &str) -> io::Result<()> {
    require_marker(cli, VARIANT_MARKER)?;
    require_marker(commands_mod, MODULE_MARKER)?;
    require_marker(commands_mod, DISPATCH_MARKER)?;

    Ok(())
}

fn require_marker(contents: &str, marker: &str) -> io::Result<()> {
    if contents.contains(marker) {
        return Ok(());
    }

    Err(io::Error::new(
        ErrorKind::InvalidData,
        format!("required Rusty generator marker is missing: {marker}"),
    ))
}

fn insert_before(contents: &str, marker: &str, addition: &str) -> io::Result<String> {
    let position = contents.find(marker).ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("required insertion marker is missing: {marker}"),
        )
    })?;

    let mut updated = String::with_capacity(contents.len() + addition.len() + 2);

    updated.push_str(&contents[..position]);
    updated.push_str(addition);
    updated.push_str("\n\n");
    updated.push_str(&contents[position..]);

    Ok(updated)
}

fn write_new(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                io::Error::new(
                    ErrorKind::AlreadyExists,
                    format!("refusing to overwrite existing command: {}", path.display()),
                )
            } else {
                error
            }
        })?;

    file.write_all(contents.as_bytes())
}

fn validate_cli_segment(value: &str) -> io::Result<()> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("invalid command segment: {value}"),
        ));
    }

    if !value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
    {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "command segments must begin with a letter",
        ));
    }

    Ok(())
}

fn validate_rust_identifier(value: &str, description: &str) -> io::Result<()> {
    let mut characters = value.chars();

    let first = characters.next().ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} cannot be empty"),
        )
    })?;

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} must begin with a letter or underscore"),
        ));
    }

    if !characters.all(|character| character.is_ascii_alphanumeric() || character == '_') {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} contains invalid characters"),
        ));
    }

    Ok(())
}
