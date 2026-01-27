pub mod sqlite;

use std::any::Any;

use async_trait::async_trait;

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

#[async_trait]
pub trait Database: Send + Sync + 'static {
    /// List tables within the database
    async fn database_tables(&self) -> anyhow::Result<Vec<DatabaseTable>>;

    /// Perform a query against the database
    async fn query(&self, query: &str) -> anyhow::Result<Vec<DatabaseRow>>;
}
