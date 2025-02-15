use omnitron_gate_common::{OmnitronError, User, UserDetails};
use poem_openapi::Object;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::Serialize;
use uuid::Uuid;

use crate::{OtpCredential, PasswordCredential, PublicKeyCredential, Role};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Object)]
#[sea_orm(table_name = "users")]
#[oai(rename = "User")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub username: String,
  pub credential_policy: serde_json::Value,
}

impl Related<super::Role::Entity> for Entity {
  fn to() -> RelationDef {
    super::UserRoleAssignment::Relation::Role.def()
  }

  fn via() -> Option<RelationDef> {
    Some(super::UserRoleAssignment::Relation::User.def().rev())
  }
}

impl Related<super::OtpCredential::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OtpCredentials.def()
  }
}

impl Related<super::PasswordCredential::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::PasswordCredentials.def()
  }
}

impl Related<super::PublicKeyCredential::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::PublicKeyCredentials.def()
  }
}

impl Related<super::ApiToken::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::ApiTokens.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter)]
#[allow(clippy::enum_variant_names)]
pub enum Relation {
  OtpCredentials,
  PasswordCredentials,
  PublicKeyCredentials,
  ApiTokens,
}

impl RelationTrait for Relation {
  fn def(&self) -> RelationDef {
    match self {
      Self::OtpCredentials => Entity::has_many(super::OtpCredential::Entity)
        .from(Column::Id)
        .to(super::OtpCredential::Column::UserId)
        .into(),
      Self::PasswordCredentials => Entity::has_many(super::PasswordCredential::Entity)
        .from(Column::Id)
        .to(super::PasswordCredential::Column::UserId)
        .into(),
      Self::PublicKeyCredentials => Entity::has_many(super::PublicKeyCredential::Entity)
        .from(Column::Id)
        .to(super::PublicKeyCredential::Column::UserId)
        .into(),
      Self::ApiTokens => Entity::has_many(super::ApiToken::Entity)
        .from(Column::Id)
        .to(super::ApiToken::Column::UserId)
        .into(),
    }
  }
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<Model> for User {
  type Error = OmnitronError;

  fn try_from(model: Model) -> Result<Self, OmnitronError> {
    Ok(User {
      id: model.id,
      username: model.username,
      credential_policy: serde_json::from_value(model.credential_policy)?,
    })
  }
}

impl Model {
  pub async fn load_details(self, db: &DatabaseConnection) -> Result<UserDetails, OmnitronError> {
    let roles: Vec<String> = self
      .find_related(Role::Entity)
      .all(db)
      .await?
      .into_iter()
      .map(Into::<omnitron_gate_common::Role>::into)
      .map(|x| x.name)
      .collect();

    let mut credentials = vec![];
    credentials.extend(
      self
        .find_related(OtpCredential::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|x| x.into()),
    );
    credentials.extend(
      self
        .find_related(PasswordCredential::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|x| x.into()),
    );
    credentials.extend(
      self
        .find_related(PublicKeyCredential::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|x| x.into()),
    );

    Ok(omnitron_gate_common::UserDetails {
      inner: self.try_into()?,
      roles,
      credentials,
    })
  }
}

impl TryFrom<User> for ActiveModel {
  type Error = OmnitronError;

  fn try_from(user: User) -> Result<Self, Self::Error> {
    Ok(Self {
      id: Set(user.id),
      username: Set(user.username),
      credential_policy: Set(serde_json::to_value(&user.credential_policy)?),
    })
  }
}
