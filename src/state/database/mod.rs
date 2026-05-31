use gpui::Entity;

use crate::{
    database::{AnySharedDatabase, DatabaseTable},
    state::{async_resource::AsyncResource, database::connection::DatabaseConnectionResource},
};

pub mod connection;
pub mod tables;

pub trait DatabaseResourceExt {
    /// Get the database entity
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>>;

    /// Get a handle to the current database connection if
    /// one is available
    fn database_connection(&self) -> Option<AnySharedDatabase>;

    /// Get the database connection resource entity
    fn database_connection_resource(&self) -> Entity<DatabaseConnectionResource>;

    /// Get a [Entity] of the derived tables async resource
    fn database_tables(&self) -> Entity<AsyncResource<Vec<DatabaseTable>>>;
}
