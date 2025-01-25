use std::time::Duration;

use anyhow::Result;
use omnitron_gate_common::helpers::fs::secure_file;
use omnitron_gate_common::{OmnitronConfig, OmnitronError, TargetOptions, TargetWebAdminOptions};
use omnitron_gate_db_entities::Target::TargetKind;
use omnitron_gate_db_entities::{LogEntry, Role, Target, TargetRoleAssignment};
use omnitron_gate_db_migrations::migrate_database;
use sea_orm::sea_query::Expr;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter,
  TransactionTrait,
};
use uuid::Uuid;

use crate::consts::{BUILTIN_ADMIN_ROLE_NAME, BUILTIN_ADMIN_TARGET_NAME};

pub async fn connect_to_db(config: &OmnitronConfig) -> Result<DatabaseConnection> {
  let mut url = url::Url::parse(&config.store.database_url.expose_secret()[..])?;
  if url.scheme() == "sqlite" {
    let path = url.path();
    let mut abs_path = config.paths_relative_to.clone();
    abs_path.push(path);
    abs_path.push("db.sqlite3");

    if let Some(parent) = abs_path.parent() {
      std::fs::create_dir_all(parent)?
    }

    url.set_path(
      abs_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert database path to string"))?,
    );

    url.set_query(Some("mode=rwc"));

    let db = Database::connect(ConnectOptions::new(url.to_string())).await?;
    db.begin().await?.commit().await?;

    secure_file(&abs_path)?;
  }

  let mut opt = ConnectOptions::new(url.to_string());
  opt
    .max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(true);

  let connection = Database::connect(opt).await?;

  migrate_database(&connection).await?;
  Ok(connection)
}

pub async fn populate_db(db: &mut DatabaseConnection, _config: &mut OmnitronConfig) -> Result<(), OmnitronError> {
  use omnitron_gate_db_entities::Session;
  use sea_orm::ActiveValue::Set;

  Session::Entity::update_many()
    .set(Session::ActiveModel {
      ended: Set(Some(chrono::Utc::now())),
      ..Default::default()
    })
    .filter(Expr::col(Session::Column::Ended).is_null())
    .exec(db)
    .await
    .map_err(OmnitronError::from)?;

  let admin_role = match Role::Entity::find()
    .filter(Role::Column::Name.eq(BUILTIN_ADMIN_ROLE_NAME))
    .all(db)
    .await?
    .first()
  {
    Some(x) => x.to_owned(),
    None => {
      let values = Role::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(BUILTIN_ADMIN_ROLE_NAME.to_owned()),
      };
      values.insert(&*db).await.map_err(OmnitronError::from)?
    }
  };

  let admin_target = match Target::Entity::find()
    .filter(Target::Column::Kind.eq(TargetKind::WebAdmin))
    .all(db)
    .await?
    .first()
  {
    Some(x) => x.to_owned(),
    None => {
      let values = Target::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(BUILTIN_ADMIN_TARGET_NAME.to_owned()),
        kind: Set(TargetKind::WebAdmin),
        options: Set(serde_json::to_value(TargetOptions::WebAdmin(TargetWebAdminOptions {})).map_err(OmnitronError::from)?),
      };

      values.insert(&*db).await.map_err(OmnitronError::from)?
    }
  };

  if TargetRoleAssignment::Entity::find()
    .filter(TargetRoleAssignment::Column::TargetId.eq(admin_target.id))
    .filter(TargetRoleAssignment::Column::RoleId.eq(admin_role.id))
    .all(db)
    .await?
    .is_empty()
  {
    let values = TargetRoleAssignment::ActiveModel {
      target_id: Set(admin_target.id),
      role_id: Set(admin_role.id),
      ..Default::default()
    };
    values.insert(&*db).await.map_err(OmnitronError::from)?;
  }

  Ok(())
}

pub async fn cleanup_db(db: &mut DatabaseConnection, retention: &Duration) -> Result<()> {
  use omnitron_gate_db_entities::Session;
  let cutoff = chrono::Utc::now() - chrono::Duration::from_std(*retention)?;

  LogEntry::Entity::delete_many()
    .filter(Expr::col(LogEntry::Column::Timestamp).lt(cutoff))
    .exec(db)
    .await?;

  Session::Entity::delete_many()
    .filter(Expr::col(Session::Column::Ended).is_not_null())
    .filter(Expr::col(Session::Column::Ended).lt(cutoff))
    .exec(db)
    .await?;

  Ok(())
}
