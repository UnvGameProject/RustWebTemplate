use crate::domain::contact::UpdateContactInput;

#[test]
fn normalizes_name_and_email_before_validation() {
    let input = UpdateContactInput::from_untrusted("  Grace Hopper  ", "  GRACE@Example.COM  ");

    let valid = input
        .validate()
        .expect("normalized contact input should be valid");

    assert_eq!(valid.name(), "Grace Hopper");
    assert_eq!(valid.email(), "grace@example.com");
}

#[test]
fn rejects_blank_name_after_normalization() {
    let result = UpdateContactInput::from_untrusted("   ", "grace@example.com").validate();

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_email() {
    let result = UpdateContactInput::from_untrusted("Grace Hopper", "not-an-email").validate();

    assert!(result.is_err());
}
