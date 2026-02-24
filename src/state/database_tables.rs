use anyhow::Context;
use gpui::{Entity, Window};

use crate::{
    database::DatabaseTable,
    state::{
        AppState, DatabaseStoreEvent,
        async_resource::{AsyncResource, AsyncResourceEntityExt},
    },
};

pub fn database_tables_resource<T: 'static>(
    window: &mut Window,
    cx: &mut gpui::Context<T>,
) -> Entity<AsyncResource<Vec<DatabaseTable>>> {
    let tables = AsyncResource::new(cx);

    let app = cx.global::<AppState>();
    let database_store = app.database_store.clone();

    let async_tables = tables.clone();

    cx.subscribe_in(
        &database_store,
        window,
        move |_this: &mut T, database_store, event, window, cx| match event {
            DatabaseStoreEvent::DatabaseChanged => {
                let database = match database_store.read(cx).database.as_ref() {
                    Some(value) => value.clone(),
                    None => {
                        async_tables.set_idle(cx);
                        return;
                    }
                };

                async_tables.load(window, cx, || async move {
                    tracing::debug!("loading database tables");
                    database
                        .database_tables()
                        .await
                        .context("failed to load database tables")
                });
            }
        },
    )
    .detach();

    tables
}
