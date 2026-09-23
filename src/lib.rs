mod app;
mod config;
mod db;
mod web;

use std::error::Error;

pub type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub async fn run() -> AppResult<()> {
    app::run().await
}

pub async fn run_migrations() -> AppResult<()> {
    let database_config = config::DatabaseConfig::load()?;

    let database = db::connect(&database_config, "topcoat-poc-migrator").await?;

    db::migrate(&database).await?;

    database.close().await;

    Ok(())
}
