pub(super) fn name(value: &str) -> String {
    value.trim().to_owned()
}

pub(super) fn email(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
