use std::path::Path;

use async_trait::async_trait;
use sqlx::{Connection, SqliteConnection, prelude::FromRow, sqlite::SqliteConnectOptions};
use tokio::sync::Mutex;

use crate::database::{Database, DatabaseTable};

pub struct SqliteDatabase {
    connection: Mutex<SqliteConnection>,
}

impl SqliteDatabase {
    pub async fn from_path(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            anyhow::bail!(
                "database path '{path}' is not a file",
                path = path.display()
            );
        }

        let options = SqliteConnectOptions::new().filename(path);

        let connection = SqliteConnection::connect_with(&options).await?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    #[allow(unused)]
    pub async fn memory() -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::new().in_memory(true);
        let connection = SqliteConnection::connect_with(&options).await?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    /// List tables within the database
    async fn database_tables(&self) -> anyhow::Result<Vec<DatabaseTable>> {
        #[derive(FromRow)]
        struct SqliteTable {
            name: String,
            sql: String,
        }

        let mut connection = self.connection.lock().await;

        let result: Vec<SqliteTable> = sqlx::query_as(
            r#"
            SELECT "name", "sql"
            FROM sqlite_master
            WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
            ORDER BY "name"
            "#,
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(result
            .into_iter()
            .map(|value| DatabaseTable {
                name: value.name,
                sql: value.sql,
            })
            .collect())
    }
}
