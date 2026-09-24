use crate::domain::contact::CreateContactInput;

#[test]
fn normalizes_name_and_email_before_validation() {
    let input = CreateContactInput::from_untrusted("  Ada Lovelace  ", "  ADA@Example.COM  ");

    let valid = input
        .validate()
        .expect("normalized contact input should be valid");

    assert_eq!(valid.name(), "Ada Lovelace");
    assert_eq!(valid.email(), "ada@example.com");
}

#[test]
fn rejects_blank_name_after_normalization() {
    let result = CreateContactInput::from_untrusted("   ", "ada@example.com").validate();

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_email() {
    let result = CreateContactInput::from_untrusted("Ada Lovelace", "not-an-email").validate();

    assert!(result.is_err());
}

#[test]
fn accepts_name_at_maximum_length() {
    let name = "a".repeat(200);

    let result = CreateContactInput::from_untrusted(&name, "ada@example.com").validate();

    assert!(result.is_ok());
}

#[test]
fn rejects_name_over_maximum_length() {
    let name = "a".repeat(201);

    let result = CreateContactInput::from_untrusted(&name, "ada@example.com").validate();

    assert!(result.is_err());
}

#[test]
fn counts_unicode_name_length_by_characters_not_bytes() {
    let name = "é".repeat(200);

    let result = CreateContactInput::from_untrusted(&name, "ada@example.com").validate();

    assert!(result.is_ok());
}
