use std::sync::Arc;

use gpui::{App, actions};

use crate::{database::sqlite::SqliteDatabase, state::AppState};

actions!(file, [NewMemoryDatabase]);

pub fn new_memory_database(_: &NewMemoryDatabase, cx: &mut App) {
    cx.spawn(async move |cx| {
        let database = match SqliteDatabase::memory().await {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(?error, "failed to connect to database");
                return;
            }
        };

        tracing::debug!("loaded database");

        if let Err(error) = cx.update_global(|global: &mut AppState, cx| {
            let database_store = global.database_store.clone();
            database_store.update(cx, |this, cx| {
                this.set_database(Some(Arc::new(database)), cx);
            })
        }) {
            tracing::error!(?error, "failed to update global state")
        }
    })
    .detach();
}
