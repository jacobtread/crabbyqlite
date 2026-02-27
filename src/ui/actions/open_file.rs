use crate::{
    database::{AnySharedDatabase, sqlite::SqliteDatabase},
    state::{AppStateExt, async_resource::AsyncResourceEntityExt},
};
use anyhow::Context;
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

    let database = cx.database();

    database.maybe_load(cx, async move || {
        let paths = match prompt_recv.await {
            Ok(Ok(Some(value))) => value,

            // Error occurred
            Ok(Err(error)) => return Err(error.context("failed to pick file")),

            // Cancelled picking the file or picked nothing
            Err(_) | Ok(Ok(None)) => return Ok(None),
        };

        let path = match paths.first() {
            Some(value) => value,
            // Picked nothing
            None => return Ok(None),
        };

        tracing::debug!(?path, "picked file for opening");

        let database = SqliteDatabase::from_path(path)
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Arc::new(database);

        tracing::debug!("loaded database");

        Ok(Some(database))
    });
}
