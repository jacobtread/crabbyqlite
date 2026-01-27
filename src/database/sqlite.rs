use std::path::Path;

use async_trait::async_trait;
use sqlx::{
    Column, ConnectOptions, Connection, Decode, Row, Sqlite, SqliteConnection, Value, ValueRef,
    prelude::FromRow,
    sqlite::{SqliteConnectOptions, SqliteValueRef},
};
use tokio::sync::Mutex;

use crate::database::{Database, DatabaseColumn, DatabaseName, DatabaseRow, DatabaseTable};

pub struct SqliteDatabase {
    name: DatabaseName,
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
            name: DatabaseName {
                primary: path
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string()),
                secondary: options.to_url_lossy().to_string(),
            },
            connection: Mutex::new(connection),
        })
    }

    #[allow(unused)]
    pub async fn memory() -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::new().in_memory(true);
        let connection = SqliteConnection::connect_with(&options).await?;
        Ok(Self {
            name: DatabaseName {
                primary: "Memory".to_string(),
                secondary: options.to_url_lossy().to_string(),
            },
            connection: Mutex::new(connection),
        })
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    fn name(&self) -> DatabaseName {
        self.name.clone()
    }

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

    async fn query(&self, query: &str) -> anyhow::Result<Vec<DatabaseRow>> {
        let mut connection = self.connection.lock().await;

        let result = sqlx::query(query).fetch_all(&mut *connection).await?;

        Ok(result
            .into_iter()
            .map(|row| {
                let columns = row.columns();

                DatabaseRow {
                    value: columns
                        .iter()
                        .map(|column| {
                            let value = row.try_get_raw(column.ordinal()).unwrap();

                            DatabaseColumn {
                                name: column.name().to_string(),
                                value: value_to_string(value),
                            }
                        })
                        .collect(),
                }
            })
            .collect())
    }
}

fn value_to_string(value: SqliteValueRef<'_>) -> String {
    if value.is_null() {
        return "NULL".to_string();
    }

    let value = value.to_owned();

    if let Ok(value) = <String as Decode<'_, Sqlite>>::decode(value.as_ref()) {
        return value;
    }

    if let Ok(value) = <&str as Decode<'_, Sqlite>>::decode(value.as_ref()) {
        return value.to_string();
    }
    if let Ok(value) = <i64 as Decode<'_, Sqlite>>::decode(value.as_ref()) {
        return value.to_string();
    }
    if let Ok(value) = <f64 as Decode<'_, Sqlite>>::decode(value.as_ref()) {
        return value.to_string();
    }

    if let Ok(value) = <bool as Decode<'_, Sqlite>>::decode(value.as_ref()) {
        return value.to_string();
    }

    if let Ok(value) = <Vec<u8> as Decode<'_, Sqlite>>::decode(value.as_ref()) {
        return format!("{:?}", value);
    }

    // Fallback: type name
    format!("<unhandled type: {}>", value.type_info())
}
