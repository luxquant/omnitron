use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use omnitron_gate_common::OmnitronConfig;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

use crate::db::{connect_to_db, populate_db};
use crate::{AuthStateStore, ConfigProvider, DatabaseConfigProvider, State};

type ConfigProviderArc = Arc<Mutex<dyn ConfigProvider + Send + 'static>>;

#[derive(Clone)]
pub struct Services {
  pub db: Arc<Mutex<DatabaseConnection>>,
  pub config: Arc<Mutex<OmnitronConfig>>,
  pub state: Arc<Mutex<State>>,
  pub config_provider: ConfigProviderArc,
  pub auth_state_store: Arc<Mutex<AuthStateStore>>,
  pub admin_token: Arc<Mutex<Option<String>>>,
}

impl Services {
  pub async fn new(mut config: OmnitronConfig, admin_token: Option<String>) -> Result<Self> {
    let mut db = connect_to_db(&config).await?;
    populate_db(&mut db, &mut config).await?;
    let db = Arc::new(Mutex::new(db));

    let config = Arc::new(Mutex::new(config));

    let config_provider = Arc::new(Mutex::new(DatabaseConfigProvider::new(&db).await));

    let auth_state_store = Arc::new(Mutex::new(AuthStateStore::new(config_provider.clone())));

    tokio::spawn({
      let auth_state_store = auth_state_store.clone();
      async move {
        loop {
          auth_state_store.lock().await.vacuum().await;
          tokio::time::sleep(Duration::from_secs(60)).await;
        }
      }
    });

    Ok(Self {
      db: db.clone(),
      config: config.clone(),
      state: State::new(&db),
      config_provider,
      auth_state_store,
      admin_token: Arc::new(Mutex::new(admin_token)),
    })
  }
}
