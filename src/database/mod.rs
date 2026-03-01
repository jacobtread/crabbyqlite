pub mod sqlite;

use std::{any::Any, rc::Rc};

use async_trait::async_trait;

/// Database naming details
#[derive(Debug, Clone)]
pub struct DatabaseName {
    /// Primary name (File name)
    pub primary: String,
    /// Alternative name (Connection String)
    pub secondary: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseTable {
    /// Name of the database table
    pub name: String,
    /// SQL used to create the database table
    pub sql: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseRow {
    pub value: Vec<DatabaseColumn>,
}

#[derive(Debug, Clone)]
pub struct DatabaseColumn {
    pub name: String,
    pub value: String,
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
    fn name(&self) -> DatabaseName;

    /// List tables within the database
    async fn database_tables(&self) -> anyhow::Result<Vec<DatabaseTable>>;

    /// Perform a query against the database
    async fn query(&self, query: &str) -> anyhow::Result<Vec<DatabaseRow>>;

    /// Query the rows of a specific table
    async fn query_table_rows(
        &self,
        query: DatabaseTableQuery,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<DatabaseRow>>;

    /// Query the total number of rows within a table
    async fn query_table_rows_count(&self, query: DatabaseTableQuery) -> anyhow::Result<i64>;
}
