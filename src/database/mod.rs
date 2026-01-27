pub mod sqlite;

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct DatabaseTable {
    /// Name of the database table
    pub name: String,
    /// SQL used to create the database table
    pub sql: String,
}

#[async_trait]
pub trait Database: Send + Sync + 'static {
    /// List tables within the database
    async fn database_tables(&self) -> anyhow::Result<Vec<DatabaseTable>>;
}
