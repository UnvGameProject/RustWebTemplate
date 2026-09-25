use std::fmt;

use argon2::{
    Argon2,
    password_hash::{Error, PasswordHasher, PasswordVerifier},
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PasswordHash(String);

impl PasswordHash {
    pub(crate) fn from_stored(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PasswordHash([REDACTED])")
    }
}

pub(crate) fn hash_password(password: &str) -> Result<PasswordHash, Error> {
    let hash = Argon2::default().hash_password(password.as_bytes())?;

    Ok(PasswordHash(hash.to_string()))
}

pub(crate) fn verify_password(password: &str, expected: &PasswordHash) -> Result<bool, Error> {
    match Argon2::default().verify_password(password.as_bytes(), expected.as_str()) {
        Ok(()) => Ok(true),

        Err(Error::PasswordInvalid) => Ok(false),

        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests;
