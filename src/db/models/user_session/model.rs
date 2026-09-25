use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UserSession {
    token_hash: [u8; 32],
    user_id: Uuid,
    expires_at: OffsetDateTime,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl UserSession {
    pub(crate) fn token_hash(&self) -> &[u8; 32] {
        &self.token_hash
    }

    pub(crate) fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub(crate) fn expires_at(&self) -> OffsetDateTime {
        self.expires_at
    }

    pub(crate) fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    pub(crate) fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}

impl fmt::Debug for UserSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UserSession")
            .field("token_hash", &"[REDACTED]")
            .field("user_id", &self.user_id)
            .field("expires_at", &self.expires_at)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

/// Database-facing representation of `public.user_sessions`.
///
/// PostgreSQL `BYTEA` decodes as `Vec<u8>`. Conversion into `UserSession`
/// re-establishes the application's compile-time 32-byte token-hash invariant.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct UserSessionRow {
    pub(super) token_hash: Vec<u8>,
    pub(super) user_id: Uuid,
    pub(super) expires_at: OffsetDateTime,
    pub(super) created_at: OffsetDateTime,
    pub(super) updated_at: OffsetDateTime,
}

#[derive(Debug)]
pub(super) struct InvalidTokenHashLength {
    actual: usize,
}

impl Display for InvalidTokenHashLength {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "stored session token hash must be 32 bytes, found {}",
            self.actual,
        )
    }
}

impl Error for InvalidTokenHashLength {}

impl TryFrom<UserSessionRow> for UserSession {
    type Error = InvalidTokenHashLength;

    fn try_from(row: UserSessionRow) -> Result<Self, Self::Error> {
        let actual = row.token_hash.len();

        let token_hash = row
            .token_hash
            .try_into()
            .map_err(|_| InvalidTokenHashLength { actual })?;

        Ok(Self {
            token_hash,
            user_id: row.user_id,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
