use std::time::Duration;

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::config::DatabaseConfig;

pub(crate) async fn connect(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::new()
        .host(config.host())
        .port(config.port())
        .username(config.user())
        .password(config.password())
        .database(config.name())
        .application_name("topcoat-poc");

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect_with(options)
        .await
}

pub(crate) async fn healthcheck(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;

    Ok(())
}