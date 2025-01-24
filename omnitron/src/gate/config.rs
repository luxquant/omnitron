#![allow(clippy::collapsible_else_if)]
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::Arc;

use anyhow::{Context, Result};
use config::{Config, Environment};
use gate_common::helpers::fs::{secure_directory, secure_file};
use gate_common::{
  HttpConfig, MySqlConfig, OmnitronConfig, OmnitronConfigStore, OmnitronError, PostgresConfig, Secret, SshConfig,
  UserPasswordCredential, UserRequireCredentialsPolicy,
};
use gate_core::consts::{BUILTIN_ADMIN_ROLE_NAME, BUILTIN_ADMIN_USERNAME};
use gate_core::Services;
use gate_db_entities::{PasswordCredential, Role, User, UserRoleAssignment};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use rcgen::generate_simple_self_signed;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::*;
use uuid::Uuid;

use crate::helpers::path_from_global;

pub(crate) async fn create_config() -> Result<()> {
  let config_path = path_from_global("omnitron.gate.config");

  if config_path.exists() {
    return Ok(());
  }

  let mut store = OmnitronConfigStore {
    http: HttpConfig {
      enable: true,
      ..Default::default()
    },
    ..Default::default()
  };

  // ---

  let data_path = path_from_global("omnitron.gate.data").canonicalize()?;
  let db_path = data_path.join("db");
  create_dir_all(&db_path)?;
  secure_directory(&db_path)?;

  store.database_url = Secret::new({
    let db_path = db_path.to_string_lossy().to_string();

    format!("sqlite:{db_path}")
  });

  store.http.enable = true;
  store.http.listen = HttpConfig::default().listen;

  store.ssh.enable = false;
  store.ssh.listen = SshConfig::default().listen;
  store.ssh.keys = data_path.join("ssh-keys").to_string_lossy().to_string();

  store.mysql.enable = false;
  store.mysql.listen = MySqlConfig::default().listen;

  store.postgres.enable = false;
  store.postgres.listen = PostgresConfig::default().listen;

  store.http.certificate = data_path.join("tls.certificate.pem").to_string_lossy().to_string();

  store.http.key = data_path.join("tls.key.pem").to_string_lossy().to_string();

  store.mysql.certificate = store.http.certificate.clone();
  store.mysql.key = store.http.key.clone();

  store.postgres.certificate = store.http.certificate.clone();
  store.postgres.key = store.http.key.clone();

  // ---

  let admin_password = Secret::new(if let Ok(admin_password) = std::env::var("OMNITRON_ADMIN_PASSWORD") {
    admin_password
  } else {
    "admin".to_string()
  });

  // ---

  let yaml = serde_yaml::to_string(&store)?;

  File::create(&config_path)?.write_all(yaml.as_bytes())?;

  let config = load_config(true)?;
  let services = Services::new(config.clone(), None).await?;
  gate_protocol_ssh::generate_host_keys(&config)?;
  gate_protocol_ssh::generate_client_keys(&config)?;

  {
    let db = services.db.lock().await;

    let admin_role = Role::Entity::find()
      .filter(Role::Column::Name.eq(BUILTIN_ADMIN_ROLE_NAME))
      .all(&*db)
      .await?
      .into_iter()
      .next()
      .ok_or_else(|| anyhow::anyhow!("Database inconsistent: no admin role"))?;

    let admin_user = match User::Entity::find()
      .filter(User::Column::Username.eq(BUILTIN_ADMIN_USERNAME))
      .all(&*db)
      .await?
      .first()
    {
      Some(x) => x.to_owned(),
      None => {
        let values = User::ActiveModel {
          id: Set(Uuid::new_v4()),
          username: Set(BUILTIN_ADMIN_USERNAME.to_owned()),
          credential_policy: Set(serde_json::to_value(None::<UserRequireCredentialsPolicy>)?),
        };
        values.insert(&*db).await.map_err(OmnitronError::from)?
      }
    };

    PasswordCredential::ActiveModel {
      user_id: Set(admin_user.id),
      id: Set(Uuid::new_v4()),
      ..UserPasswordCredential::from_password(&admin_password).into()
    }
    .insert(&*db)
    .await?;

    if UserRoleAssignment::Entity::find()
      .filter(UserRoleAssignment::Column::UserId.eq(admin_user.id))
      .filter(UserRoleAssignment::Column::RoleId.eq(admin_role.id))
      .all(&*db)
      .await?
      .is_empty()
    {
      let values = UserRoleAssignment::ActiveModel {
        user_id: Set(admin_user.id),
        role_id: Set(admin_role.id),
        ..Default::default()
      };
      values.insert(&*db).await.map_err(OmnitronError::from)?;
    }
  }

  {
    let cert = generate_simple_self_signed(vec!["omnitron.local".to_string(), "localhost".to_string()])?;

    let certificate_path = config.paths_relative_to.join(&config.store.http.certificate);
    let key_path = config.paths_relative_to.join(&config.store.http.key);
    std::fs::write(&certificate_path, cert.serialize_pem()?)?;
    std::fs::write(&key_path, cert.serialize_private_key_pem())?;
    secure_file(&certificate_path)?;
    secure_file(&key_path)?;
  }

  Ok(())
}

pub(crate) fn load_config(secure: bool) -> Result<OmnitronConfig> {
  let path = path_from_global("omnitron.gate.config");
  let store: serde_yaml::Value = Config::builder()
    .add_source(config::File::from(path.as_path()))
    .add_source(Environment::with_prefix("OMNITRON"))
    .build()
    .context("Could not load config")?
    .try_deserialize()
    .context("Could not parse YAML")?;
  if secure {
    secure_file(path.as_path()).context("Could not secure config")?;
  }

  let store: OmnitronConfigStore = serde_yaml::from_value(store).context("Could not parse YAML")?;

  let config = OmnitronConfig {
    store,
    paths_relative_to: path_from_global("omnitron.gate.base"),
  };

  config.validate();
  Ok(config)
}

pub(crate) fn watch_config(config: Arc<Mutex<OmnitronConfig>>) -> Result<broadcast::Receiver<()>> {
  let path = path_from_global("omnitron.gate.config");
  let (tx, mut rx) = mpsc::channel(16);
  let mut watcher = recommended_watcher(move |res| {
    let _ = tx.blocking_send(res);
  })?;
  watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

  let (tx2, rx2) = broadcast::channel(16);
  tokio::spawn(async move {
    let _watcher = watcher; // avoid dropping the watcher
    loop {
      match rx.recv().await {
        Some(Ok(event)) => {
          if event.kind.is_modify() {
            match load_config(false) {
              Ok(new_config) => {
                *(config.lock().await) = new_config;
                let _ = tx2.send(());
                info!("Reloaded config");
              }
              Err(error) => error!(?error, "Failed to reload config"),
            }
          }
        }
        Some(Err(error)) => error!(?error, "Failed to watch config"),
        None => {
          error!("Config watch failed");
          break;
        }
      }
    }

    #[allow(unreachable_code)]
    Ok::<_, anyhow::Error>(())
  });

  Ok(rx2)
}
