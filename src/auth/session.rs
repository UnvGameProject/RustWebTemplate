use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use sqlx::PgPool;
use time::OffsetDateTime;
use topcoat::{
    context::{Cx, app_context},
    session::{self as topcoat_session, Session, TokenHash},
};
use uuid::Uuid;

use crate::db::models::user_session::UserSession;

pub(crate) type SessionResult<T> = Result<T, SessionError>;

#[derive(Debug)]
pub(crate) enum SessionError {
    Framework(topcoat::Error),
    Storage(sqlx::Error),
}

impl Display for SessionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Framework(_) => formatter.write_str("session framework operation failed"),

            Self::Storage(_) => formatter.write_str("session storage operation failed"),
        }
    }
}

impl Error for SessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Framework(error) => error.chain().next(),
            Self::Storage(error) => Some(error),
        }
    }
}

impl From<topcoat::Error> for SessionError {
    fn from(error: topcoat::Error) -> Self {
        Self::Framework(error)
    }
}

impl From<sqlx::Error> for SessionError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error)
    }
}

/// Start a fresh Topcoat session and bind its persisted representation
/// to the authenticated application user.
///
/// If persistence fails after Topcoat issued the client token, make a
/// best-effort attempt to discard that token before returning the error.
pub(crate) async fn start(cx: &Cx, user_id: Uuid) -> SessionResult<UserSession> {
    let framework_session = topcoat_session::start(cx).await?;

    let (token_hash, expires_at) = session_parts(framework_session);

    match UserSession::create(pool(cx), &token_hash, user_id, expires_at).await {
        Ok(session) => Ok(session),

        Err(error) => {
            let _ = topcoat_session::stop(cx).await;

            Err(SessionError::Storage(error))
        }
    }
}

/// Resolve the current Topcoat token against application-owned session
/// storage.
///
/// A valid client token is not sufficient for authentication: the
/// corresponding database record must also exist and be unexpired.
pub(crate) async fn current(cx: &Cx) -> SessionResult<Option<UserSession>> {
    let Some(token_hash) = topcoat_session::token_hash(cx).await? else {
        return Ok(None);
    };

    let token_hash = token_hash_bytes(&token_hash);

    Ok(UserSession::find_active(pool(cx), &token_hash).await?)
}

/// Extend the current authenticated session.
///
/// The application database remains authoritative: an expired or missing
/// persisted session cannot be refreshed.
pub(crate) async fn refresh(cx: &Cx) -> SessionResult<Option<UserSession>> {
    if current(cx).await?.is_none() {
        return Ok(None);
    }

    let Some(framework_session) = topcoat_session::refresh(cx).await? else {
        return Ok(None);
    };

    let (token_hash, expires_at) = session_parts(framework_session);

    match UserSession::update_expiry(pool(cx), &token_hash, expires_at).await {
        Ok(Some(session)) => Ok(Some(session)),

        Ok(None) => {
            topcoat_session::stop(cx).await?;

            Ok(None)
        }

        Err(error) => {
            // Topcoat already refreshed the client token. If application
            // persistence fails, fail closed by attempting to discard it.
            let _ = topcoat_session::stop(cx).await;

            Err(SessionError::Storage(error))
        }
    }
}

/// Replace the current session token after a privilege change or other
/// rotation event.
///
/// The application row is re-keyed from Topcoat's explicitly revoked hash
/// to the replacement hash.
pub(crate) async fn rotate(cx: &Cx) -> SessionResult<Option<UserSession>> {
    if current(cx).await?.is_none() {
        return Ok(None);
    }

    let Some(rotation) = topcoat_session::rotate(cx).await? else {
        return Ok(None);
    };

    let revoked = token_hash_bytes(&rotation.revoked);

    let (replacement, expires_at) = session_parts(rotation.session);

    match UserSession::replace_token(pool(cx), &revoked, &replacement, expires_at).await {
        Ok(Some(session)) => Ok(Some(session)),

        Ok(None) => {
            // The persisted session disappeared or expired between
            // resolution and rotation. The new client token must not
            // survive without an application-owned session record.
            topcoat_session::stop(cx).await?;

            Ok(None)
        }

        Err(error) => {
            // Topcoat has already rotated the client-side token. Attempt
            // to revoke both sides rather than leaving the previous
            // database session usable after a failed re-key.
            let _ = UserSession::delete(pool(cx), &revoked).await;

            let _ = topcoat_session::stop(cx).await;

            Err(SessionError::Storage(error))
        }
    }
}

/// Revoke the application-owned session before discarding the client token.
///
/// Doing the database deletion first means a successful logout cannot leave
/// a server-side session usable by another copy of the same raw token.
pub(crate) async fn stop(cx: &Cx) -> SessionResult<bool> {
    let Some(token_hash) = topcoat_session::token_hash(cx).await? else {
        topcoat_session::stop(cx).await?;

        return Ok(false);
    };

    let token_hash = token_hash_bytes(&token_hash);

    let deleted = UserSession::delete(pool(cx), &token_hash).await?;

    topcoat_session::stop(cx).await?;

    Ok(deleted)
}

fn pool(cx: &Cx) -> &PgPool {
    app_context::<PgPool>(cx)
}

fn session_parts(session: Session) -> ([u8; 32], OffsetDateTime) {
    (
        token_hash_bytes(&session.token_hash),
        OffsetDateTime::from(session.expires_at),
    )
}

fn token_hash_bytes(token_hash: &TokenHash) -> [u8; 32] {
    **token_hash
}

#[cfg(test)]
mod tests;
