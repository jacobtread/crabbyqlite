use std::{any::Any, path::Path, rc::Rc};

use async_trait::async_trait;
use tokio_rusqlite::{Connection, OpenFlags, params, types::ValueRef};

use tokio::sync::{Mutex, MutexGuard};

use crate::database::{
    Database, DatabaseColumn, DatabaseOptions, DatabaseRow, DatabaseTable, DatabaseTableQuery,
};

pub struct SqliteDatabase {
    connection: Mutex<Connection>,
    options: DatabaseOptions,
}

#[derive(Default)]
pub struct SqliteDatabaseOptions {
    pub readonly: bool,
    pub key: Option<String>,
}

impl SqliteDatabase {
    pub async fn from_path(path: &Path, db_options: SqliteDatabaseOptions) -> anyhow::Result<Self> {
        if !path.exists() {
            anyhow::bail!(
                "database path '{path}' is not a file",
                path = path.display()
            );
        }

        let readonly = db_options.readonly;
        let encrypted = db_options.key.is_some();

        let mut flags = OpenFlags::default();
        if readonly {
            flags.remove(OpenFlags::SQLITE_OPEN_READ_WRITE);
            flags.insert(OpenFlags::SQLITE_OPEN_READ_ONLY);
        }

        let connection = Connection::open_with_flags(path, flags).await?;

        if let Some(key) = db_options.key {
            connection
                .call(move |connection| connection.pragma_update(None, "key", key))
                .await?;
        }

        let path = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Ok(Self {
            connection: Mutex::new(connection),
            options: DatabaseOptions {
                path,
                readonly,
                encrypted,
            },
        })
    }

    #[allow(unused)]
    pub async fn memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().await?;
        Ok(Self {
            connection: Mutex::new(connection),
            options: DatabaseOptions {
                path: ":memory:".to_string(),
                ..Default::default()
            },
        })
    }

    /// Directly acquire the underlying database connection
    #[allow(unused)]
    async fn connection(&self) -> MutexGuard<'_, Connection> {
        self.connection.lock().await
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn options(&self) -> DatabaseOptions {
        self.options.clone()
    }

    async fn database_tables(&self) -> anyhow::Result<Vec<DatabaseTable>> {
        let connection = self.connection.lock().await;

        let result = connection
            .call(|connection| {
                let mut statement = connection.prepare(
                    r#"
            SELECT "name", "sql"
            FROM sqlite_master
            WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
            ORDER BY "name"
            "#,
                )?;

                let results = statement.query_map(params![], |row| {
                    Ok(DatabaseTable {
                        name: row.get(0)?,
                        sql: row.get(1)?,
                    })
                })?;

                let mut result: Vec<DatabaseTable> = Vec::new();

                for row_result in results {
                    result.push(row_result?);
                }

                Ok::<_, tokio_rusqlite::rusqlite::Error>(result)
            })
            .await?;

        Ok(result)
    }

    async fn query(&self, query: &str) -> anyhow::Result<Vec<DatabaseRow>> {
        let connection = self.connection.lock().await;
        let query = query.to_string();

        let result = connection
            .call(move |connection| {
                let mut statement = connection.prepare(&query)?;

                // Collect the available column names
                let column_names: Vec<String> = statement
                    .column_names()
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect();

                let results = statement.query_map(params![], move |row| {
                    let mut columns: Vec<DatabaseColumn> = Vec::with_capacity(column_names.len());

                    for (i, column_name) in column_names.iter().enumerate() {
                        let value = row.get_ref(i)?;
                        let value = value_to_string(value);

                        columns.push(DatabaseColumn {
                            name: column_name.to_string(),
                            value,
                        });
                    }

                    Ok(DatabaseRow { value: columns })
                })?;

                let mut result: Vec<DatabaseRow> = Vec::new();

                for row_result in results {
                    result.push(row_result?);
                }

                Ok::<_, tokio_rusqlite::rusqlite::Error>(result)
            })
            .await?;

        Ok(result)
    }

    async fn query_table_rows(
        &self,
        query: DatabaseTableQuery,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<DatabaseRow>> {
        self.query(
            format!(
                "SELECT * FROM {table} LIMIT {limit} OFFSET {offset}",
                table = query.table,
            )
            .as_str(),
        )
        .await
    }

    async fn query_table_rows_count(&self, query: DatabaseTableQuery) -> anyhow::Result<i64> {
        let connection = self.connection.lock().await;
        let sql = format!("SELECT COUNT(*)  FROM {table}", table = query.table);

        let count: i64 = connection
            .call(move |connection| connection.query_one(&sql, params![], |row| row.get(0)))
            .await?;

        Ok(count)
    }
}

fn value_to_string(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => "NULL".to_string(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(items) => {
            let value = match std::str::from_utf8(items) {
                Ok(value) => value,
                Err(err) => return format!("Failed to decode Text column: {err}"),
            };

            format!("{:?}", value)
        }
        ValueRef::Blob(items) => format!("{:?}", items),
    }
}
