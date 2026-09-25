use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::password::PasswordHash;

/// Persisted non-secret representation of one user.
///
/// Password hashes deliberately do not belong to the ordinary `User`
/// representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) email: String,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

/// Complete database row used internally for schema verification and
/// credential loading.
///
/// This type intentionally does not derive `Debug` because it contains the
/// encoded password hash as an ordinary database string.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct UserRow {
    pub(super) id: Uuid,
    pub(super) email: String,
    pub(super) password_hash: String,
    pub(super) created_at: OffsetDateTime,
    pub(super) updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserCredentials {
    user: User,
    password_hash: PasswordHash,
}

impl UserCredentials {
    pub(crate) fn user(&self) -> &User {
        &self.user
    }

    pub(crate) fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }
}

impl UserRow {
    pub(super) fn into_credentials(self) -> UserCredentials {
        UserCredentials {
            user: User {
                id: self.id,
                email: self.email,
                created_at: self.created_at,
                updated_at: self.updated_at,
            },
            password_hash: PasswordHash::from_stored(self.password_hash),
        }
    }
}
