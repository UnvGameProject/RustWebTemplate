use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
};

static MIGRATOR: Migrator = sqlx::migrate!();

pub(crate) async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await?;

    // The runtime application must never be able to alter SQLx's
    // migration history, even though default table privileges are
    // granted to topcoat_app for application tables.
    sqlx::query(
        "REVOKE ALL PRIVILEGES
         ON TABLE public._sqlx_migrations
         FROM topcoat_app",
    )
    .execute(pool)
    .await?;

    Ok(())
}
