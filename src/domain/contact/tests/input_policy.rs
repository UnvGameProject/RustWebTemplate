use crate::domain::contact::CreateContactInput;

#[test]
fn rejects_html_markup_in_plain_text_name() {
    let result =
        CreateContactInput::from_untrusted("<b>Bold Person</b>", "bold@example.com").validate();

    assert!(result.is_err());
}

#[test]
fn rejects_script_markup_in_plain_text_name() {
    let result =
        CreateContactInput::from_untrusted("<script>alert(1)</script>", "script@example.com")
            .validate();

    assert!(result.is_err());
}

#[test]
fn rejects_uppercase_html_markup_in_plain_text_name() {
    let result = CreateContactInput::from_untrusted(
        "<SCRIPT>alert(1)</SCRIPT>",
        "uppercase-script@example.com",
    )
    .validate();

    assert!(result.is_err());
}

#[test]
fn rejects_html_markup_with_attributes_in_plain_text_name() {
    let result = CreateContactInput::from_untrusted(
        "<script src=x></script>",
        "script-attributes@example.com",
    )
    .validate();

    assert!(result.is_err());
}

#[test]
fn accepts_non_html_special_characters_in_plain_text_name() {
    for name in [
        "O'Connor",
        r#""quoted""#,
        r"back\slash",
        "$(rm -rf /)",
        "José Álvarez",
        "李小龍",
    ] {
        let result = CreateContactInput::from_untrusted(name, "plain@example.com").validate();

        assert!(
            result.is_ok(),
            "expected plain-text name to be accepted: {name:?}"
        );
    }
}

#[test]
fn preserves_command_shaped_text_as_domain_data() {
    let input = CreateContactInput::from_untrusted("  $(rm -rf /)  ", "command@example.com")
        .validate()
        .expect("command-shaped text should still be valid plain text");

    assert_eq!(input.name(), "$(rm -rf /)");
}

#[test]
fn trims_without_corrupting_unicode_name_content() {
    let input = CreateContactInput::from_untrusted(
        "  O’Connor / José Álvarez / 李小龍  ",
        "unicode@example.com",
    )
    .validate()
    .expect("Unicode name content should remain valid");

    assert_eq!(input.name(), "O’Connor / José Álvarez / 李小龍");
}
