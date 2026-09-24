use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Component, Path, PathBuf},
};

use convert_case::{Case, Casing};

use crate::{cli::TestArgs, commands::CommandResult, project::Project};

pub(crate) fn run(args: TestArgs) -> CommandResult {
    let project = Project::discover()?;
    let module_path = normalize_module_path(&args.module)?;

    if args.view {
        scaffold_view_test(project.root(), &module_path)?;
    } else {
        let raw_name = args.name.as_deref().ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                "test name is required unless --view is used",
            )
        })?;

        let test_name = normalize_test_name(raw_name)?;

        scaffold_module_test(project.root(), &module_path, &test_name)?;
    }

    Ok(())
}

fn scaffold_module_test(
    project_root: &Path,
    module_path: &Path,
    test_name: &str,
) -> io::Result<()> {
    let module_dir = project_root.join("src").join(module_path);

    if !module_dir.is_dir() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("target module does not exist: {}", module_dir.display()),
        ));
    }

    let module_mod = module_dir.join("mod.rs");

    if !module_mod.is_file() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("target module has no mod.rs: {}", module_mod.display()),
        ));
    }

    let tests_dir = module_dir.join("tests");
    let tests_mod = tests_dir.join("mod.rs");
    let test_file = tests_dir.join(format!("{test_name}.rs"));

    if test_file.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "refusing to overwrite existing test file: {}",
                test_file.display()
            ),
        ));
    }

    fs::create_dir_all(&tests_dir)?;

    let module_display = module_path.to_string_lossy().replace('\\', "/");

    let contents = format!(
        "\
//! Tests for `{module_display}`.
//!
//! Add test cases for `{test_name}` here.
"
    );

    write_new(&test_file, &contents)?;

    if let Err(error) = ensure_parent_tests_module(&module_mod) {
        let _ = fs::remove_file(&test_file);
        return Err(error);
    }

    if let Err(error) = ensure_test_module(&tests_mod, test_name) {
        let _ = fs::remove_file(&test_file);
        return Err(error);
    }

    println!("Created test:");
    println!("  {}", test_file.display());
    println!();
    println!("Module registration verified:");
    println!("  {}", module_mod.display());
    println!("  {}", tests_mod.display());

    Ok(())
}

fn scaffold_view_test(project_root: &Path, module_path: &Path) -> io::Result<()> {
    let src_dir = project_root.join("src");

    let source_file = src_dir.join(module_path).with_extension("rs");

    if !source_file.is_file() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!(
                "target view module does not exist: {}",
                source_file.display()
            ),
        ));
    }

    let tests_dir = src_dir.join(module_path);
    let conflicting_mod = tests_dir.join("mod.rs");

    if conflicting_mod.exists() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!(
                "view-associated test directory may not contain mod.rs: {}",
                conflicting_mod.display()
            ),
        ));
    }

    let test_file = tests_dir.join("tests.rs");

    if test_file.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "refusing to overwrite existing view test file: {}",
                test_file.display()
            ),
        ));
    }

    fs::create_dir_all(&tests_dir)?;

    let module_display = module_path.to_string_lossy().replace('\\', "/");

    let contents = format!(
        "\
//! Tests associated with `src/{module_display}.rs`.
//!
//! Add tests for this view-backed module here.
"
    );

    write_new(&test_file, &contents)?;

    if let Err(error) = ensure_parent_tests_module(&source_file) {
        let _ = fs::remove_file(&test_file);
        return Err(error);
    }

    println!("Created view-associated test:");
    println!("  {}", test_file.display());
    println!();
    println!("Module registration verified:");
    println!("  {}", source_file.display());

    Ok(())
}

fn normalize_module_path(raw: &str) -> io::Result<PathBuf> {
    let raw = raw.trim();

    if raw.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "module path cannot be empty",
        ));
    }

    let path = Path::new(raw);

    if path.is_absolute() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "module path must be relative to src/",
        ));
    }

    let mut normalized = PathBuf::new();
    let mut first = true;

    for component in path.components() {
        match component {
            Component::CurDir => {}

            Component::Normal(segment) => {
                let segment = segment.to_str().ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidInput, "module path must be valid UTF-8")
                })?;

                // Accept either:
                //
                // db/models/contact
                //
                // or:
                //
                // src/db/models/contact
                if first && segment == "src" {
                    first = false;
                    continue;
                }

                validate_identifier(segment, "module path segment")?;

                normalized.push(segment);
                first = false;
            }

            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "module path may not escape src/",
                ));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "module path must identify a module below src/",
        ));
    }

    Ok(normalized)
}

fn normalize_test_name(raw: &str) -> io::Result<String> {
    let name = raw.trim().to_case(Case::Snake);

    validate_identifier(&name, "test name")?;

    Ok(name)
}

fn validate_identifier(value: &str, description: &str) -> io::Result<()> {
    if value.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} cannot be empty"),
        ));
    }

    let mut characters = value.chars();

    let first = characters.next().expect("checked non-empty");

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} must begin with a letter or underscore"),
        ));
    }

    if !characters.all(|character| character.is_ascii_alphanumeric() || character == '_') {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} may contain only letters, numbers, and underscores"),
        ));
    }

    if is_rust_keyword(value) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} may not be a reserved Rust keyword"),
        ));
    }

    Ok(())
}

fn ensure_parent_tests_module(mod_file: &Path) -> io::Result<()> {
    let contents = fs::read_to_string(mod_file)?;

    if has_module_declaration(&contents, "tests") {
        return Ok(());
    }

    let updated = append_block(&contents, "#[cfg(test)]\nmod tests;");

    fs::write(mod_file, updated)
}

fn ensure_test_module(mod_file: &Path, test_name: &str) -> io::Result<()> {
    let contents = if mod_file.exists() {
        fs::read_to_string(mod_file)?
    } else {
        String::new()
    };

    if has_module_declaration(&contents, test_name) {
        return Ok(());
    }

    let declaration = format!("mod {test_name};");
    let updated = append_block(&contents, &declaration);

    fs::write(mod_file, updated)
}

fn has_module_declaration(contents: &str, module: &str) -> bool {
    let private = format!("mod {module};");
    let public = format!("pub mod {module};");
    let crate_public = format!("pub(crate) mod {module};");

    contents.lines().any(|line| {
        let line = line.trim();

        line == private || line == public || line == crate_public
    })
}

fn append_block(contents: &str, block: &str) -> String {
    let mut updated = contents.trim_end().to_owned();

    if !updated.is_empty() {
        updated.push_str("\n\n");
    }

    updated.push_str(block);
    updated.push('\n');

    updated
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
                    format!("refusing to overwrite existing file: {}", path.display()),
                )
            } else {
                error
            }
        })?;

    file.write_all(contents.as_bytes())
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "yield"
    )
}

#[cfg(test)]
mod tests;
