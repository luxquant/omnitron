use sea_orm::Schema;
use sea_orm_migration::prelude::*;

pub mod session {
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    use crate::m00001_create_ticket::ticket;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sessions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub target_snapshot: Option<String>,
        pub username: Option<String>,
        pub remote_address: String,
        pub started: DateTimeUtc,
        pub ended: Option<DateTimeUtc>,
        pub ticket_id: Option<Uuid>,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Ticket,
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Ticket => Entity::belongs_to(ticket::Entity)
                    .from(Column::TicketId)
                    .to(ticket::Column::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .into(),
            }
        }
    }

    impl Related<ticket::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Ticket.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00002_create_session"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);
        manager
            .create_table(schema.create_table_from_entity(session::Entity))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(session::Entity).to_owned())
            .await
    }
}
