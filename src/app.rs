use std::error::Error;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt},
    runtime::RouterBuilderRuntimeExt,
};

use crate::{
    config::DatabaseConfig,
    db,
};

pub(crate) async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let database_config = DatabaseConfig::load()?;

    let database = db::connect(&database_config).await?;
    db::healthcheck(&database).await?;

    let router = Router::builder()
        .runtime()
        .discover()
        .app_context(database)
        .assets(AssetBundle::load().expect("Topcoat asset bundle was not found"))
        .build();

    topcoat::start(router).await?;

    Ok(())
}