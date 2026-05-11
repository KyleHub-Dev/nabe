use std::sync::Arc;

use nabe_adguard_adapter::{AdguardAdapter, Credentials};

use crate::{config::Config, db::Database};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Arc<Database>,
    pub adguard: AdguardAdapter,
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

        let credentials = match (
            config.adguard_username.clone(),
            config.adguard_password.clone(),
        ) {
            (Some(username), Some(password)) => Some(Credentials::new(username, password)),
            _ => None,
        };
        let adguard = AdguardAdapter::new(config.adguard_base_url.clone(), credentials);

        Ok(Self {
            config,
            db,
            adguard,
        })
    }
}
