use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{normalize_module_path, scaffold_module_test, scaffold_view_test};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);

        let root = env::temp_dir().join(format!("rusty-cli-{label}-{}-{id}", process::id(),));

        fs::create_dir_all(root.join("src"))
            .expect("temporary project src directory should be created");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn view_test_creates_associated_tests_file_and_registers_module() {
    let project = TempProject::new("view-create");

    let source_file = project.root().join("src/web/pages/contacts.rs");

    fs::create_dir_all(
        source_file
            .parent()
            .expect("source file should have a parent"),
    )
    .expect("source directory should be created");

    fs::write(&source_file, "fn marker() {}\n").expect("source file should be written");

    let module_path =
        normalize_module_path("web/pages/contacts").expect("view module path should normalize");

    scaffold_view_test(project.root(), &module_path).expect("view test should scaffold");

    let tests_dir = project.root().join("src/web/pages/contacts");

    assert!(tests_dir.join("tests.rs").is_file());

    assert!(
        !tests_dir.join("mod.rs").exists(),
        "--view must not create contacts/mod.rs",
    );

    assert!(
        !tests_dir.join("tests/mod.rs").exists(),
        "--view must not create contacts/tests/mod.rs",
    );

    let source = fs::read_to_string(&source_file).expect("source file should remain readable");

    assert!(source.contains("#[cfg(test)]\nmod tests;"));
}

#[test]
fn view_test_refuses_existing_tests_file_without_duplicate_registration() {
    let project = TempProject::new("view-repeat");

    let source_file = project.root().join("src/web/pages/contacts.rs");

    fs::create_dir_all(
        source_file
            .parent()
            .expect("source file should have a parent"),
    )
    .expect("source directory should be created");

    fs::write(&source_file, "fn marker() {}\n").expect("source file should be written");

    let module_path =
        normalize_module_path("web/pages/contacts").expect("view module path should normalize");

    scaffold_view_test(project.root(), &module_path).expect("first scaffold should succeed");

    let error = scaffold_view_test(project.root(), &module_path)
        .expect_err("second scaffold should refuse overwrite");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists);

    let source = fs::read_to_string(&source_file).expect("source file should remain readable");

    assert_eq!(
        source.matches("#[cfg(test)]\nmod tests;").count(),
        1,
        "test module registration must not be duplicated",
    );
}

#[test]
fn view_test_rejects_missing_source_module() {
    let project = TempProject::new("view-missing");

    let module_path =
        normalize_module_path("web/pages/missing").expect("view module path should normalize");

    let error = scaffold_view_test(project.root(), &module_path)
        .expect_err("missing view source should fail");

    assert_eq!(error.kind(), ErrorKind::NotFound);
}

#[test]
fn module_path_rejects_parent_traversal() {
    let error =
        normalize_module_path("../../outside").expect_err("parent traversal must be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
}

#[test]
fn normal_make_test_scaffolding_remains_unchanged() {
    let project = TempProject::new("normal-regression");

    let module_dir = project.root().join("src/domain/contact");

    fs::create_dir_all(&module_dir).expect("module directory should be created");

    let module_mod = module_dir.join("mod.rs");

    fs::write(&module_mod, "pub(crate) mod create;\n").expect("module file should be written");

    let module_path =
        normalize_module_path("domain/contact").expect("module path should normalize");

    scaffold_module_test(project.root(), &module_path, "update")
        .expect("normal test scaffold should succeed");

    assert!(module_dir.join("tests/update.rs").is_file());

    let tests_mod =
        fs::read_to_string(module_dir.join("tests/mod.rs")).expect("tests/mod.rs should exist");

    assert!(tests_mod.contains("mod update;"));

    let parent = fs::read_to_string(&module_mod).expect("parent module should remain readable");

    assert!(parent.contains("#[cfg(test)]\nmod tests;"));
}
