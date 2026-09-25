use clap::Parser;

use super::{Cli, Command};

#[test]
fn make_test_accepts_existing_module_form() {
    let cli = Cli::try_parse_from(["rusty", "make:test", "domain/contact", "create"])
        .expect("normal make:test syntax should parse");

    let Command::MakeTest(args) = cli.command else {
        panic!("expected make:test command");
    };

    assert_eq!(args.module, "domain/contact");
    assert_eq!(args.name.as_deref(), Some("create"));
    assert!(!args.view);
}

#[test]
fn make_test_view_accepts_module_without_test_name() {
    let cli = Cli::try_parse_from(["rusty", "make:test", "--view", "web/pages/contacts"])
        .expect("--view syntax should parse without a test name");

    let Command::MakeTest(args) = cli.command else {
        panic!("expected make:test command");
    };

    assert_eq!(args.module, "web/pages/contacts");
    assert_eq!(args.name, None);
    assert!(args.view);
}

#[test]
fn make_test_without_view_requires_test_name() {
    let result = Cli::try_parse_from(["rusty", "make:test", "domain/contact"]);

    assert!(result.is_err());
}

#[test]
fn make_test_view_rejects_test_name() {
    let result = Cli::try_parse_from([
        "rusty",
        "make:test",
        "--view",
        "web/pages/contacts",
        "extra",
    ]);

    assert!(result.is_err());
}

#[test]
fn audit_command_parses() {
    let cli = Cli::try_parse_from(["rusty", "audit"]).expect("audit command should parse");

    assert!(matches!(cli.command, Command::Audit));
}
