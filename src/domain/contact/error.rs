use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub(crate) enum ContactError {
    EmailAlreadyExists,
    NotFound,
    Storage(Box<dyn Error + Send + Sync>),
}

impl ContactError {
    pub(crate) fn storage(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Storage(Box::new(error))
    }
}

impl Display for ContactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmailAlreadyExists => {
                formatter.write_str("a contact with this email already exists")
            }
            Self::NotFound => formatter.write_str("contact not found"),
            Self::Storage(_) => formatter.write_str("contact storage operation failed"),
        }
    }
}

impl Error for ContactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            Self::EmailAlreadyExists | Self::NotFound => None,
        }
    }
}
