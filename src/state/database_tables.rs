use anyhow::Context;
use gpui::Entity;

use crate::{
    database::{AnySharedDatabase, DatabaseTable},
    state::{
        AppStateExt, QueryExecutedEvent,
        async_resource::{AsyncResource, AsyncResourceEntityExt},
    },
};

pub fn database_tables_resource<T: 'static>(
    cx: &mut gpui::Context<T>,
) -> Entity<AsyncResource<Vec<DatabaseTable>>> {
    let tables = AsyncResource::new(cx);
    let database = cx.database();

    // Refresh the database tables when the database changes
    let async_tables = tables.clone();
    cx.observe(&database, move |_view, database, cx| {
        load_database_tables(database, &async_tables, cx);
    })
    .detach();

    // Refresh the database tables when the user performs a query
    let async_tables = tables.clone();
    cx.subscribe(
        &database,
        move |_view, database, _event: &QueryExecutedEvent, cx| {
            load_database_tables(database, &async_tables, cx);
        },
    )
    .detach();

    tables
}

fn load_database_tables<T: 'static>(
    database: Entity<AsyncResource<AnySharedDatabase>>,
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
