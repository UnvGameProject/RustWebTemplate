use garde::{Unvalidated, Validate};

/// Untrusted input used when creating a contact.
///
/// Construction performs field-specific normalization, but the value remains
/// untrusted until Garde converts it from `Unvalidated<Self>` to `Valid<Self>`.
#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub(crate) struct CreateContactInput {
    #[garde(length(chars, min = 1, max = 200))]
    name: String,

    #[garde(email, length(chars, min = 3, max = 320))]
    email: String,
}

impl CreateContactInput {
    /// Normalize untrusted HTTP/form input without marking it as valid.
    ///
    /// Validation must still succeed before this value can cross the
    /// application trust boundary.
    pub(crate) fn from_untrusted(
        name: impl AsRef<str>,
        email: impl AsRef<str>,
    ) -> Unvalidated<Self> {
        Unvalidated::new(Self {
            name: normalize_name(name.as_ref()),
            email: normalize_email(email.as_ref()),
        })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}

fn normalize_name(value: &str) -> String {
    value.trim().to_owned()
}

fn normalize_email(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
