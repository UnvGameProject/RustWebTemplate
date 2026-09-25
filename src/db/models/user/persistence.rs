use garde::Valid;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::password::PasswordHash,
    domain::user::{CreateUserInput, UserError},
};

use super::{User, UserCredentials, model::UserRow};

impl User {
    /// Persist a previously validated user and previously hashed password.
    ///
    /// Neither an unvalidated email nor a raw password can cross this
    /// persistence boundary through this API.
    pub(crate) async fn create(
        pool: &PgPool,
        input: &Valid<CreateUserInput>,
        password_hash: PasswordHash,
    ) -> Result<Self, UserError> {
        let id = Uuid::new_v4();
        let password_hash = password_hash.into_string();

        sqlx::query_as!(
            User,
            r#"
            INSERT INTO public.users (
                id,
                email,
                password_hash
            )
            VALUES ($1, $2, $3)
            RETURNING
                id,
                email,
                created_at,
                updated_at
            "#,
            id,
            input.email(),
            password_hash,
        )
        .fetch_one(pool)
        .await
        .map_err(map_user_error)
    }

    pub(crate) async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                email,
                created_at,
                updated_at
            FROM public.users
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(pool)
        .await
    }

    pub(crate) async fn find_credentials_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<UserCredentials>, sqlx::Error> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT
                id,
                email,
                password_hash,
                created_at,
                updated_at
            FROM public.users
            WHERE lower(email) = lower($1::text)
            "#,
            email,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(UserRow::into_credentials))
    }
}

fn map_user_error(error: sqlx::Error) -> UserError {
    match &error {
        sqlx::Error::Database(database_error)
            if database_error.is_unique_violation()
                && database_error.constraint() == Some("users_email_unique") =>
        {
            UserError::EmailAlreadyExists
        }

        _ => UserError::storage(error),
    }
}

/// Compile-time assertion that `UserRow` represents the complete shape of
/// `public.users`.
#[allow(dead_code)]
fn schema_shape_contract() {
    let _ = sqlx::query_as!(
        UserRow,
        r#"
        SELECT *
        FROM public.users
        "#
    );
}

/// Compile-time assertion that PostgreSQL nullability and types map exactly
/// to the complete internal user row.
#[allow(dead_code)]
async fn schema_nullability_contract(pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            email,
            password_hash,
            created_at,
            updated_at
        FROM public.users
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    let _ = UserRow {
        id: row.id,
        email: row.email,
        password_hash: row.password_hash,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(())
}
