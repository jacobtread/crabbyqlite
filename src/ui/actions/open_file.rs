use crate::{database::sqlite::SqliteDatabase, state::AppState};
use gpui::{App, PathPromptOptions, actions};
use std::sync::Arc;

actions!(file, [OpenFile]);

pub fn open_file(_: &OpenFile, cx: &mut App) {
    let prompt_recv = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        multiple: false,
        directories: false,
        prompt: Some("SQLite database files (*.db, *.sqlite, *.sqlite3, *.db3)".into()),
    });

    cx.spawn(async |cx| {
        let paths = match prompt_recv.await {
            Ok(Ok(Some(value))) => value,

            // Error occurred
            Ok(Err(error)) => {
                tracing::error!(?error, "failed to pick file");
                return;
            }

            // Cancelled picking the file or picked nothing
            Err(_) | Ok(Ok(None)) => return,
        };

        let path = match paths.first() {
            Some(value) => value,
            // Picked nothing
            None => return,
        };

        tracing::debug!(?path, "picked file for opening");

        let database = match SqliteDatabase::from_path(path).await {
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
