use garde::{Unvalidated, Validate};

use crate::input::Normalize;

#[derive(Debug, Validate, Normalize)]
pub(crate) struct CreateContactInput {
    #[normalize(trim)]
    #[garde(
        length(chars, min = 1, max = 200),
        custom(crate::input::validation::plain_text)
    )]
    name: String,

    #[normalize(trim, ascii_lowercase)]
    #[garde(email, length(chars, min = 3, max = 320))]
    email: String,
}

impl CreateContactInput {
    pub(crate) fn from_untrusted(
        name: impl AsRef<str>,
        email: impl AsRef<str>,
    ) -> Unvalidated<Self> {
        let mut input = Self {
            name: name.as_ref().to_owned(),
            email: email.as_ref().to_owned(),
        };

        input.normalize();

        Unvalidated::new(input)
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}
