use sqlx::{Pool, Sqlite};
use tracing::instrument;

use crate::internal::settings::Settings;

pub mod players;

#[instrument]
pub async fn init(settings: &Settings) -> Pool<Sqlite> {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&settings.database.filename)
                .create_if_missing(true),
        )
        .await
        .expect("Could not connect to database")
}

#[instrument]
pub async fn migrate(database: &Pool<Sqlite>) {
    sqlx::migrate!("./migrations")
        .run(database)
        .await
        .expect("Could not run database migrations");
}
