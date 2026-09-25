use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub(crate) enum UserError {
    EmailAlreadyExists,
    Storage(Box<dyn Error + Send + Sync>),
}

impl UserError {
    pub(crate) fn storage(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Storage(Box::new(error))
    }
}

impl Display for UserError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmailAlreadyExists => {
                formatter.write_str("a user with this email already exists")
            }

            Self::Storage(_) => formatter.write_str("user storage operation failed"),
        }
    }
}

impl Error for UserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            Self::EmailAlreadyExists => None,
        }
    }
}
