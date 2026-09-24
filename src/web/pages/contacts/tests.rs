use super::validation_outcome;
use crate::domain::contact::CreateContactInput;

#[test]
fn validation_outcome_identifies_name_error() {
    let report = CreateContactInput::from_untrusted("   ", "ada@example.com")
        .validate()
        .expect_err("blank name should fail validation");

    assert_eq!(validation_outcome(&report), "validation_name");
}

#[test]
fn validation_outcome_identifies_email_error() {
    let report = CreateContactInput::from_untrusted("Ada Lovelace", "not-an-email")
        .validate()
        .expect_err("invalid email should fail validation");

    assert_eq!(validation_outcome(&report), "validation_email");
}

#[test]
fn validation_outcome_identifies_both_field_errors() {
    let report = CreateContactInput::from_untrusted("   ", "not-an-email")
        .validate()
        .expect_err("both fields should fail validation");

    assert_eq!(validation_outcome(&report), "validation_both");
}
