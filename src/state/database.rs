use anyhow::Context;
use gpui::{App, AppContext, Entity, EventEmitter, SharedString};

use crate::{
    database::{AnySharedDatabase, DatabaseTable},
    state::async_resource::{AsyncResource, AsyncResourceEntityExt},
};

/// Event emitted when the sql query executor performs a query
#[derive(Clone)]
pub struct QueryExecutedEvent {
    #[allow(unused)]
    pub query: SharedString,
}

pub trait DatabaseResourceExt {
    /// Get the database entity
    fn database(&self) -> Entity<AsyncResource<AnySharedDatabase>>;

    /// Get a handle to the current database connection if
    /// one is available
    fn database_connection(&self) -> Option<AnySharedDatabase>;

    /// Get a [Entity] of the derived tables async resource
    fn database_tables(&self) -> Entity<AsyncResource<Vec<DatabaseTable>>>;
}

pub struct DatabaseResource {
    /// Inner async loaded database entity
    database: Entity<AsyncResource<AnySharedDatabase>>,

    /// Async resource for the database tables
    /// within the currently loaded database
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,
}

impl DatabaseResource {
    pub fn new(cx: &mut App) -> Entity<DatabaseResource> {
        cx.new(|cx| {
            let database = AsyncResource::new(cx);
            let tables = AsyncResource::new(cx);
            Self::apply_table_derives(&database, cx);
            DatabaseResource { database, tables }
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

    pub fn database_tables(
        this: &Entity<Self>,
        cx: &App,
    ) -> Entity<AsyncResource<Vec<DatabaseTable>>> {
        this.read(cx).tables.clone()
    }

    /// Applies the required listeners to derive the tables state
    /// from the database state
    fn apply_table_derives(
        database: &Entity<AsyncResource<AnySharedDatabase>>,
        cx: &mut gpui::Context<Self>,
    ) {
        // Changes to the database cause the tables to be re-fetched
        cx.observe(
            database,
            move |view: &mut DatabaseResource, database, cx| {
                Self::load_database_tables(&database, &view.tables, cx);
            },
        )
        .detach();

        // User queries against the database cause the tables to be re-fetched
        cx.subscribe(
            database,
            move |view, database, _event: &QueryExecutedEvent, cx| {
                Self::load_database_tables(&database, &view.tables, cx);
            },
        )
        .detach();
    }

    /// Loads the database tables from the current value of the database entity
    /// and updates the tables value to begin loading the new tables
    fn load_database_tables<T: 'static>(
        database: &Entity<AsyncResource<AnySharedDatabase>>,
        async_tables: &Entity<AsyncResource<Vec<DatabaseTable>>>,
        cx: &mut gpui::Context<T>,
    ) {
        let database = match database.read(cx) {
            AsyncResource::Loaded(value) => value.clone(),
            _ => {
                async_tables.set_idle(cx);
                return;
            }
        };

        async_tables.load(cx, || async move {
            tracing::debug!("loading database tables");
            database
                .database_tables()
                .await
                .context("failed to load database tables")
        });
    }
}

impl EventEmitter<QueryExecutedEvent> for AsyncResource<AnySharedDatabase> {}
