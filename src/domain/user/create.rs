use garde::{Unvalidated, Validate};

use crate::input::Normalize;

#[derive(Debug, Validate, Normalize)]
pub(crate) struct CreateUserInput {
    #[normalize(trim, ascii_lowercase)]
    #[garde(email, length(chars, min = 3, max = 320))]
    email: String,
}

impl CreateUserInput {
    pub(crate) fn from_untrusted(email: impl AsRef<str>) -> Unvalidated<Self> {
        let mut input = Self {
            email: email.as_ref().to_owned(),
        };

        input.normalize();

        Unvalidated::new(input)
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}
