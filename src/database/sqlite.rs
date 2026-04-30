use std::{any::Any, path::Path, rc::Rc};

use async_trait::async_trait;
use gpui::SharedString;
use itertools::Itertools;
use tokio_rusqlite::{Connection, OpenFlags, params, rusqlite, types::ValueRef};

use tokio::sync::{Mutex, MutexGuard};

use crate::database::{
    Database, DatabaseOptions, DatabaseQueryResult, DatabaseRow, DatabaseTable,
    DatabaseTableColumn, DatabaseTableQuery,
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

fn query_table_columns(
    connection: &mut rusqlite::Connection,
    table_name: &str,
) -> Result<Vec<DatabaseTableColumn>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"SELECT "name", "type", "notnull", "pk"  FROM pragma_table_info(?1) ORDER BY "name""#,
    )?;

    let results: Vec<DatabaseTableColumn> = statement
        .query_map(params![table_name], |row| {
            Ok(DatabaseTableColumn {
                name: row.get(0)?,
                column_type: row.get(1)?,
                not_null: row.get(2)?,
                primary_key: row.get(3)?,
            })
        })?
        .try_collect()?;

    Ok(results)
}

fn query_tables(
    connection: &mut rusqlite::Connection,
) -> Result<Vec<DatabaseTable>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT "name", "sql"
        FROM sqlite_master
        WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
        ORDER BY "name"
        "#,
    )?;

    let results: Vec<DatabaseTable> = statement
        .query_map(params![], |row| {
            Ok(DatabaseTable {
                name: row.get(0)?,
                sql: row.get(1)?,
                columns: Vec::new(),
            })
        })?
        .try_collect()?;

    Ok(results)
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
                let mut tables: Vec<DatabaseTable> = query_tables(connection)?;

                for table in &mut tables {
                    let columns = query_table_columns(connection, &table.name)?;
                    table.columns = columns;
                }

                Ok::<_, rusqlite::Error>(tables)
            })
            .await?;

        Ok(result)
    }

    async fn query(&self, query: &str) -> anyhow::Result<DatabaseQueryResult> {
        let connection = self.connection.lock().await;
        let query = query.to_string();

        let result = connection
            .call(move |connection| {
                let mut statement = connection.prepare(&query)?;

                // Collect the available column names
                let column_names: Vec<SharedString> = statement
                    .column_names()
                    .into_iter()
                    .map(|value| value.into())
                    .collect();

                let column_count = statement.column_count();

                let results = statement.query_map(params![], |row| {
                    let mut values = Vec::new();

                    for i in 0..column_count {
                        let value = row.get_ref(i)?;
                        let value = value_to_string(value);
                        values.push(value.into());
                    }

                    Ok(DatabaseRow { values })
                })?;

                let mut rows: Vec<DatabaseRow> = Vec::new();

                for row_result in results {
                    rows.push(row_result?);
                }

                Ok::<_, rusqlite::Error>(DatabaseQueryResult { column_names, rows })
            })
            .await?;

        Ok(result)
    }

    async fn query_table_rows(
        &self,
        query: DatabaseTableQuery,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<DatabaseQueryResult> {
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
