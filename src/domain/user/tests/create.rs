use crate::domain::user::CreateUserInput;

#[test]
fn normalizes_email_before_validation() {
    let input = CreateUserInput::from_untrusted("  ADA@EXAMPLE.COM  ".to_owned())
        .validate()
        .expect("normalized email should be valid");

    assert_eq!(input.email(), "ada@example.com");
}

#[test]
fn rejects_invalid_email() {
    let result = CreateUserInput::from_untrusted("not-an-email".to_owned()).validate();

    assert!(result.is_err());
}

#[test]
fn rejects_email_over_maximum_length() {
    let local = "a".repeat(310);
    let email = format!("{local}@example.com");

    let result = CreateUserInput::from_untrusted(email).validate();

    assert!(result.is_err());
}
