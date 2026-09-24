use garde::{Unvalidated, Validate};

use super::normalize;

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub(crate) struct UpdateContactInput {
    #[garde(length(chars, min = 1, max = 200))]
    name: String,

    #[garde(email, length(chars, min = 3, max = 320))]
    email: String,
}

impl UpdateContactInput {
    pub(crate) fn from_untrusted(
        name: impl AsRef<str>,
        email: impl AsRef<str>,
    ) -> Unvalidated<Self> {
        Unvalidated::new(Self {
            name: normalize::name(name.as_ref()),
            email: normalize::email(email.as_ref()),
        })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}
