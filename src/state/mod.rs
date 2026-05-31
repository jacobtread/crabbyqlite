use gpui::{App, Context, Entity, Global};

use crate::{
    database::AnySharedDatabase,
    state::{
        async_resource::AsyncResource,
        database::{DatabaseResource, DatabaseResourceExt},
    },
};

pub mod async_resource;
pub mod database;

pub struct AppState {
    pub database: Entity<DatabaseResource>,
}

impl Global for AppState {}

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            database: DatabaseResource::new(cx),
        }
    }
}

impl DatabaseResourceExt for App {
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>> {
        let app_state = self.global::<AppState>();
        DatabaseResource::database(&app_state.database, self)
    }

    fn database_connection(&self) -> Option<AnySharedDatabase> {
        let app_state = self.global::<AppState>();
        DatabaseResource::database_connection(&app_state.database, self)
    }

    fn database_tables(&self) -> Entity<AsyncResource<Vec<crate::database::DatabaseTable>>> {
        let app_state = self.global::<AppState>();
        DatabaseResource::database_tables(&app_state.database, self)
    }
}

impl<'a, T> DatabaseResourceExt for Context<'a, T> {
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>> {
        let app_state = self.global::<AppState>();
        DatabaseResource::database(&app_state.database, self)
    }

    fn database_connection(&self) -> Option<AnySharedDatabase> {
        let app_state = self.global::<AppState>();
        DatabaseResource::database_connection(&app_state.database, self)
    }

    fn database_tables(&self) -> Entity<AsyncResource<Vec<crate::database::DatabaseTable>>> {
        let app_state = self.global::<AppState>();
        DatabaseResource::database_tables(&app_state.database, self)
    }
}
