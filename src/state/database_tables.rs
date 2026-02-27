use anyhow::Context;
use gpui::Entity;

use crate::{
    database::DatabaseTable,
    state::{
        AppState,
        async_resource::{AsyncResource, AsyncResourceEntityExt},
    },
};

pub fn database_tables_resource<T: 'static>(
    cx: &mut gpui::Context<T>,
) -> Entity<AsyncResource<Vec<DatabaseTable>>> {
    let tables = AsyncResource::new(cx);

    let app = cx.global::<AppState>();
    let database = app.database.clone();

    let async_tables = tables.clone();

    cx.observe(&database, move |_view, database, cx| {
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
    })
    .detach();

    tables
}
