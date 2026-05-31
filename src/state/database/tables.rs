use anyhow::Context;
use gpui::{App, AppContext, Entity, Subscription};

use crate::{
    database::{AnySharedDatabase, DatabaseTable},
    state::{
        async_resource::{AsyncResource, AsyncResourceEntityExt},
        database::connection::{DatabaseConnectionResource, QueryExecutedEvent},
    },
};

pub struct DatabaseTablesResource {
    /// Async resource for the database tables
    /// within the currently loaded database
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

    /// Subscriptions attached to this derived resource
    /// (Database entity and query event)
    _subscriptions: (Subscription, Subscription),
}

impl DatabaseTablesResource {
    pub fn derive(
        cx: &mut App,
        database: Entity<DatabaseConnectionResource>,
    ) -> Entity<DatabaseTablesResource> {
        cx.new(move |cx| {
            let tables = AsyncResource::new(cx);

            // Changes to the database cause the tables to be re-fetched
            let database_connection_entity = database.read(cx).database.clone();
            let entity_subscription = cx.observe(
                &database_connection_entity,
                move |view: &mut DatabaseTablesResource, database, cx| {
                    Self::load_database_tables(&database, &view.tables, cx);
                },
            );

            // User queries against the database cause the tables to be re-fetched
            let event_subscription = cx.subscribe(
                &database,
                move |view, database, _event: &QueryExecutedEvent, cx| {
                    let database = database.read(cx).database.clone();
                    Self::load_database_tables(&database, &view.tables, cx);
                },
            );

            DatabaseTablesResource {
                tables,
                _subscriptions: (entity_subscription, event_subscription),
            }
        })
    }

    pub fn database_tables(
        this: &Entity<Self>,
        cx: &App,
    ) -> Entity<AsyncResource<Vec<DatabaseTable>>> {
        this.read(cx).tables.clone()
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
