use sea_orm_migration::prelude::*;
use omnitron_db_migrations::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
