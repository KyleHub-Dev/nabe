use std::sync::Arc;

use crate::{adguard::AdguardClient, config::Config, db::Database};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Arc<Database>,
    pub adguard: AdguardClient,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db = Arc::new(Database::open(&config.database_path).await?);
        db.migrate().await?;
        db.ensure_engine_instance(
            "Cloud DNS",
            "adguard-home",
            config.adguard_base_url.as_str(),
        )
        .await?;

        let adguard = AdguardClient::new(
            config.adguard_base_url.clone(),
            config.adguard_username.clone(),
            config.adguard_password.clone(),
        );

        Ok(Self {
            config,
            db,
            adguard,
        })
    }
}
