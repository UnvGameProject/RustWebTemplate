pub(super) fn validation_outcome(report: &garde::Report) -> &'static str {
    let name_invalid = garde::select!(report, name).next().is_some();
    let email_invalid = garde::select!(report, email).next().is_some();

    match (name_invalid, email_invalid) {
        (true, true) => "validation_both",
        (true, false) => "validation_name",
        (false, true) => "validation_email",
        (false, false) => "validation",
    }
}
