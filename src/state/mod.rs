use gpui::{App, Context, Entity, Global};

use crate::{
    database::AnySharedDatabase,
    state::{
        async_resource::AsyncResource,
        database::{
            DatabaseResourceExt, connection::DatabaseConnectionResource,
            tables::DatabaseTablesResource,
        },
    },
};

pub mod async_resource;
pub mod database;

pub struct AppState {
    pub database: Entity<DatabaseConnectionResource>,
    pub tables: Entity<DatabaseTablesResource>,
}

impl Global for AppState {}

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        let database = DatabaseConnectionResource::new(cx);
        let tables = DatabaseTablesResource::derive(cx, database.clone());
        Self { database, tables }
    }
}

impl DatabaseResourceExt for App {
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>> {
        let app_state = self.global::<AppState>();
        DatabaseConnectionResource::database(&app_state.database, self)
    }

    fn database_connection(&self) -> Option<AnySharedDatabase> {
        let app_state = self.global::<AppState>();
        DatabaseConnectionResource::database_connection(&app_state.database, self)
    }

    fn database_connection_resource(&self) -> Entity<DatabaseConnectionResource> {
        let app_state = self.global::<AppState>();
        app_state.database.clone()
    }

    fn database_tables(&self) -> Entity<AsyncResource<Vec<crate::database::DatabaseTable>>> {
        let app_state = self.global::<AppState>();
        DatabaseTablesResource::database_tables(&app_state.tables, self)
    }
}

impl<'a, T> DatabaseResourceExt for Context<'a, T> {
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>> {
        let app_state = self.global::<AppState>();
        DatabaseConnectionResource::database(&app_state.database, self)
    }

    fn database_connection(&self) -> Option<AnySharedDatabase> {
        let app_state = self.global::<AppState>();
        DatabaseConnectionResource::database_connection(&app_state.database, self)
    }

    fn database_connection_resource(&self) -> Entity<DatabaseConnectionResource> {
        let app_state = self.global::<AppState>();
        app_state.database.clone()
    }

    fn database_tables(&self) -> Entity<AsyncResource<Vec<crate::database::DatabaseTable>>> {
        let app_state = self.global::<AppState>();
        DatabaseTablesResource::database_tables(&app_state.tables, self)
    }
}
