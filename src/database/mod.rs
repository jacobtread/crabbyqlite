pub mod sqlite;

use std::{any::Any, rc::Rc};

use async_trait::async_trait;
use gpui::SharedString;

#[derive(Debug, Clone)]
pub struct DatabaseTable {
    /// Name of the database table
    pub name: String,
    /// SQL used to create the database table
    pub sql: String,
}

#[derive(Debug, Clone, Default)]
pub struct DatabaseOptions {
    /// Path to the database file
    pub path: String,

    /// Whether the db is readonly
    pub readonly: bool,

    /// Whether the db is encrypted
    pub encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct DatabaseQueryResult {
    pub column_names: Vec<SharedString>,
    pub rows: Vec<DatabaseRow>,
}

#[derive(Debug, Clone)]
pub struct DatabaseRow {
    pub values: Vec<SharedString>,
}

#[derive(Debug, Clone)]
pub struct DatabaseTableQuery {
    pub table: String,
}

pub type AnySharedDatabase = Rc<dyn Database>;

#[async_trait]
pub trait Database: Send + Sync + 'static {
    /// Cast the database to an Any type
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;

    /// Get the name of the database
    fn options(&self) -> DatabaseOptions;

    /// List tables within the database
    async fn database_tables(&self) -> anyhow::Result<Vec<DatabaseTable>>;

    /// Perform a query against the database
    async fn query(&self, query: &str) -> anyhow::Result<DatabaseQueryResult>;

    /// Query the rows of a specific table
    async fn query_table_rows(
        &self,
        query: DatabaseTableQuery,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<DatabaseQueryResult>;

    /// Query the total number of rows within a table
    async fn query_table_rows_count(&self, query: DatabaseTableQuery) -> anyhow::Result<i64>;
}
