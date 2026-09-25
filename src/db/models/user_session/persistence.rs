use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{UserSession, model::UserSessionRow};

impl UserSession {
    pub(crate) async fn create(
        pool: &PgPool,
        token_hash: &[u8; 32],
        user_id: Uuid,
        expires_at: OffsetDateTime,
    ) -> Result<Self, sqlx::Error> {
        let row = sqlx::query_as!(
            UserSessionRow,
            r#"
            INSERT INTO public.user_sessions (
                token_hash,
                user_id,
                expires_at
            )
            VALUES ($1, $2, $3)
            RETURNING
                token_hash,
                user_id,
                expires_at,
                created_at,
                updated_at
            "#,
            &token_hash[..],
            user_id,
            expires_at,
        )
        .fetch_one(pool)
        .await?;

        decode_row(row)
    }

    /// Resolve a session only when its database expiry has not passed.
    pub(crate) async fn find_active(
        pool: &PgPool,
        token_hash: &[u8; 32],
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as!(
            UserSessionRow,
            r#"
            SELECT
                token_hash,
                user_id,
                expires_at,
                created_at,
                updated_at
            FROM public.user_sessions
            WHERE token_hash = $1
              AND expires_at > now()
            "#,
            &token_hash[..],
        )
        .fetch_optional(pool)
        .await?;

        row.map(decode_row).transpose()
    }

    /// Update the persisted expiry for sliding-session refresh.
    pub(crate) async fn update_expiry(
        pool: &PgPool,
        token_hash: &[u8; 32],
        expires_at: OffsetDateTime,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as!(
            UserSessionRow,
            r#"
            UPDATE public.user_sessions
            SET
                expires_at = $2,
                updated_at = now()
            WHERE token_hash = $1
            AND expires_at > now()
            RETURNING
                token_hash,
                user_id,
                expires_at,
                created_at,
                updated_at
            "#,
            &token_hash[..],
            expires_at,
        )
        .fetch_optional(pool)
        .await?;

        row.map(decode_row).transpose()
    }

    /// Atomically re-key an existing session during token rotation.
    pub(crate) async fn replace_token(
        pool: &PgPool,
        revoked: &[u8; 32],
        replacement: &[u8; 32],
        expires_at: OffsetDateTime,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as!(
            UserSessionRow,
            r#"
            UPDATE public.user_sessions
            SET
                token_hash = $2,
                expires_at = $3,
                updated_at = now()
            WHERE token_hash = $1
            AND expires_at > now()
            RETURNING
                token_hash,
                user_id,
                expires_at,
                created_at,
                updated_at
            "#,
            &revoked[..],
            &replacement[..],
            expires_at,
        )
        .fetch_optional(pool)
        .await?;

        row.map(decode_row).transpose()
    }

    pub(crate) async fn delete(pool: &PgPool, token_hash: &[u8; 32]) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM public.user_sessions
            WHERE token_hash = $1
            "#,
            &token_hash[..],
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() != 0)
    }
}

fn decode_row(row: UserSessionRow) -> Result<UserSession, sqlx::Error> {
    UserSession::try_from(row).map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

/// Compile-time assertion that the database row representation stays aligned
/// with the complete `public.user_sessions` table.
#[allow(dead_code)]
fn schema_shape_contract() {
    let _ = sqlx::query_as!(
        UserSessionRow,
        r#"
        SELECT *
        FROM public.user_sessions
        "#
    );
}

/// Compile-time assertion for exact PostgreSQL types and nullability.
#[allow(dead_code)]
async fn schema_nullability_contract(pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            token_hash,
            user_id,
            expires_at,
            created_at,
            updated_at
        FROM public.user_sessions
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    let _ = UserSessionRow {
        token_hash: row.token_hash,
        user_id: row.user_id,
        expires_at: row.expires_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(())
}
