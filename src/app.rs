use std::error::Error;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Router, RouterBuilderDiscoverExt},
    runtime::RouterBuilderRuntimeExt,
    session::{RouterBuilderSessionExt, SessionConfig},
};

use crate::{config::DatabaseConfig, db};

pub(crate) async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let database_config = DatabaseConfig::load()?;

    let database = db::connect(&database_config, "topcoat-poc").await?;
    db::healthcheck(&database).await?;

    let router = Router::builder()
        .cookies()
        .sessions(SessionConfig::default())
        .runtime()
        .discover()
        .app_context(database)
        .assets(AssetBundle::load().expect("Topcoat asset bundle was not found"))
        .build();

    topcoat::start(router).await?;

    Ok(())
}
