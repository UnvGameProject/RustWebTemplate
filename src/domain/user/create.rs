use garde::{Unvalidated, Validate};

use super::normalize;

#[derive(Debug, Validate)]
pub(crate) struct CreateUserInput {
    #[garde(email, length(chars, min = 3, max = 320))]
    email: String,
}

impl CreateUserInput {
    pub(crate) fn from_untrusted(email: String) -> Unvalidated<Self> {
        Unvalidated::new(Self {
            email: normalize::email(&email),
        })
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}
