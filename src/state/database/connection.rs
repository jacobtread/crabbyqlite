use crate::{database::AnySharedDatabase, state::async_resource::AsyncResource};
use gpui::{App, AppContext, Entity, EventEmitter, SharedString};

/// Event emitted when the sql query executor performs a query
#[derive(Clone)]
pub struct QueryExecutedEvent {
    #[allow(unused)]
    pub query: SharedString,
}

pub struct DatabaseConnectionResource {
    /// Inner async loaded database entity
    pub database: Entity<AsyncResource<AnySharedDatabase>>,
}

impl DatabaseConnectionResource {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let database = AsyncResource::new(cx);
            DatabaseConnectionResource { database }
        })
    }

    pub fn database(this: &Entity<Self>, cx: &App) -> Entity<AsyncResource<AnySharedDatabase>> {
        this.read(cx).database.clone()
    }

    pub fn database_connection(this: &Entity<Self>, cx: &App) -> Option<AnySharedDatabase> {
        match this.read(cx).database.read(cx) {
            AsyncResource::Loaded(value) => Some(value.clone()),
            _ => None,
        }
    }
}

impl EventEmitter<QueryExecutedEvent> for DatabaseConnectionResource {}
