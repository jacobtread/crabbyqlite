use gpui::{App, Entity, Global};

use crate::{database::AnySharedDatabase, state::async_resource::AsyncResource};

pub mod async_resource;
pub mod database_tables;

pub struct AppState {
    pub database: Entity<AsyncResource<AnySharedDatabase>>,
}

impl Global for AppState {}

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            database: AsyncResource::new(cx),
        }
    }
}

pub trait AppStateExt {
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>>;

    fn current_database(&self) -> Option<AnySharedDatabase>;
}

impl AppStateExt for App {
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>> {
        let app_state = self.global::<AppState>();
        app_state.database.clone()
    }

    fn current_database(&self) -> Option<AnySharedDatabase> {
        match self.database().read(self) {
            AsyncResource::Loaded(value) => Some(value.clone()),
            _ => None,
        }
    }
}
