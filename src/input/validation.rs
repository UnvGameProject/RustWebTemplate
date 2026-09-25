pub(crate) fn plain_text(value: &str, _context: &()) -> garde::Result {
    if ammonia::is_html(value) {
        return Err(garde::Error::new(
            "HTML markup is not allowed in this field",
        ));
    }

    Ok(())
}
