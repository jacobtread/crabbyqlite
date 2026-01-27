use std::sync::Arc;

use gpui::{Context, Entity, EventEmitter, Global};

use crate::database::Database;

pub struct AppState {
    pub database_store: Entity<DatabaseStore>,
}

impl Global for AppState {}

type AnySharedDatabase = Arc<dyn Database>;

#[derive(Clone)]
pub enum DatabaseStoreEvent {
    /// Event emitted when the current database is changed
    DatabaseChanged,
}

impl EventEmitter<DatabaseStoreEvent> for DatabaseStore {}

#[derive(Default)]
pub struct DatabaseStore {
    pub database: Option<AnySharedDatabase>,
}

impl DatabaseStore {
    pub fn set_database(&mut self, database: Option<Arc<dyn Database>>, cx: &mut Context<Self>) {
        self.database = database.clone();
        cx.emit(DatabaseStoreEvent::DatabaseChanged);
    }
}
