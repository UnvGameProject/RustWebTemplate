use garde::Valid;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::contact::CreateContactInput;

use super::Contact;

impl Contact {
    /// Persist a previously validated contact creation command.
    ///
    /// Accepting `Valid<CreateContactInput>` prevents unvalidated contact input
    /// from crossing this persistence boundary.
    pub(crate) async fn create(
        pool: &PgPool,
        input: &Valid<CreateContactInput>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            Contact,
            r#"
        INSERT INTO public.contacts (
            id,
            name,
            email
        )
        VALUES ($1, $2, $3)
        RETURNING
            id,
            name,
            email,
            created_at,
            updated_at
        "#,
            id,
            input.name(),
            input.email(),
        )
        .fetch_one(pool)
        .await
    }

    /// Fetch one contact by primary key.
    pub(crate) async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Contact,
            r#"
            SELECT
                id,
                name,
                email,
                created_at,
                updated_at
            FROM public.contacts
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Fetch all contacts in a deterministic order.
    pub(crate) async fn find_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Contact,
            r#"
            SELECT
                id,
                name,
                email,
                created_at,
                updated_at
            FROM public.contacts
            ORDER BY created_at DESC, id DESC
            "#
        )
        .fetch_all(pool)
        .await
    }

    /// Count all persisted contacts.
    pub(crate) async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM public.contacts
            "#
        )
        .fetch_one(pool)
        .await
    }
}

/// Compile-time assertion that `Contact` represents the complete row shape
/// of `public.contacts`.
///
/// `SELECT *` is intentional here. Unlike the runtime queries above, this
/// query exists specifically so an added/removed/renamed/type-changed column
/// causes the Rust model contract to fail during SQLx query analysis.
/// Compile-time assertion that `Contact` has exactly the fields returned
/// by the complete `public.contacts` row.
///
/// This catches added, removed, renamed, or incompatible fields.
#[allow(dead_code)]
fn schema_shape_contract() {
    let _ = sqlx::query_as!(
        Contact,
        r#"
        SELECT *
        FROM public.contacts
        "#
    );
}

/// Compile-time assertion that database nullability maps exactly to the
/// Rust model rather than merely being SQLx-compatible.
///
/// `query!()` derives concrete field types from PostgreSQL metadata.
/// Assigning those values into `Contact` then requires Rust's exact
/// assignment rules to succeed.
#[allow(dead_code)]
async fn schema_nullability_contract(pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            email,
            created_at,
            updated_at
        FROM public.contacts
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    let _ = Contact {
        id: row.id,
        name: row.name,
        email: row.email,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(())
}
